using System;
using System.Diagnostics;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
using System.Windows.Forms;

namespace HiveMindLauncher
{
    internal static class Program
    {
        private const string Host = "127.0.0.1";
        private const int FirstPort = 8099;
        private const int LastPort = 8199;
        private const string Edge = @"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe";
        private const string Chrome = @"C:\Program Files\Google\Chrome\Application\chrome.exe";
        private const string KeyEntropy = "HiveMind.ProviderKey.v1";

        [STAThread]
        private static void Main()
        {
            try
            {
                string server = ResolveServer();
                if (server == null)
                {
                    MessageBox.Show(
                        "HiveMind's server executable was not found.\n\nPlace hiverouter.exe next to HiveMind.exe, or build the project first.",
                        "HiveMind",
                        MessageBoxButtons.OK,
                        MessageBoxIcon.Error);
                    return;
                }

                string data = Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                    "HiveMind");
                Directory.CreateDirectory(data);
                string launchToken = LoadOrCreateLaunchToken(data);

                int port = FindUsablePort();
                if (port == 0)
                {
                    MessageBox.Show(
                        "No free loopback port was found between " + FirstPort + " and " + LastPort + ".",
                        "HiveMind",
                        MessageBoxButtons.OK,
                        MessageBoxIcon.Error);
                    return;
                }
                Process serverProcess = null;
                if (!PortOpen(port))
                {
                    string workspaceRoot = ResolveWorkspaceRoot();
                    var start = new ProcessStartInfo
                    {
                        FileName = server,
                        Arguments = "--bind " + Host + ":" + port + " --data-dir \"" + data + "\" --workspace-root \"" + workspaceRoot + "\"",
                        WorkingDirectory = workspaceRoot,
                        UseShellExecute = false,
                        CreateNoWindow = true,
                        // Never redirect without concurrent readers: a chatty child can
                        // fill a pipe and deadlock before health is reachable.
                        RedirectStandardOutput = false,
                        RedirectStandardError = false,
                    };
                    ApplyProviderKey(start);
                    start.EnvironmentVariables["HIVEROUTER_LAUNCH_TOKEN"] = launchToken;
                    serverProcess = Process.Start(start);
                    if (serverProcess == null || !WaitForHealth(port, 60000))
                    {
                        if (serverProcess != null && !serverProcess.HasExited)
                        {
                            serverProcess.Kill();
                        }
                        MessageBox.Show(
                            "HiveMind started but its local server did not become ready within 60 seconds.",
                            "HiveMind",
                            MessageBoxButtons.OK,
                            MessageBoxIcon.Error);
                        return;
                    }
                }

                string profile = Path.Combine(data, "window");
                Directory.CreateDirectory(profile);
                string browserPath = File.Exists(Edge) ? Edge : Chrome;
                var browser = new ProcessStartInfo
                {
                    FileName = browserPath,
                    Arguments = "--app=http://" + Host + ":" + port
                        + "/?launch=" + launchToken
                        + " --window-size=1440,900 --user-data-dir=\"" + profile + "\"",
                    UseShellExecute = false,
                };
                Process window = Process.Start(browser);
                if (window != null)
                {
                    var openedAt = DateTime.UtcNow;
                    window.WaitForExit();
                    // If Edge handed the URL to an already-running HiveMind window,
                    // its new process exits immediately; in that case the existing
                    // window still owns the server and it must not be killed.
                    var lifetime = DateTime.UtcNow - openedAt;
                    if (serverProcess != null && lifetime.TotalSeconds >= 3 && !serverProcess.HasExited)
                    {
                        serverProcess.Kill();
                    }
                }
                else if (serverProcess != null && !serverProcess.HasExited)
                {
                    serverProcess.Kill();
                }
            }
            catch (Exception error)
            {
                MessageBox.Show(
                    "HiveMind could not start:\n\n" + error.Message,
                    "HiveMind",
                    MessageBoxButtons.OK,
                    MessageBoxIcon.Error);
            }
        }

        /// <summary>
        /// Loads the provider key from a DPAPI-encrypted per-user file and
        /// injects it into the server child's environment. The key is never
        /// written to source, a command line, a log, or stdout/stderr.
        /// </summary>
        private static string LoadOrCreateLaunchToken(string data)
        {
            var file = Path.Combine(data, "launch.token");
            try
            {
                if (File.Exists(file))
                {
                    var existing = File.ReadAllText(file).Trim();
                    if (existing.Length >= 16)
                    {
                        return existing;
                    }
                }
            }
            catch
            {
                // A damaged token file is replaced below.
            }
            var token = Guid.NewGuid().ToString("N");
            File.WriteAllText(file, token, new UTF8Encoding(false));
            return token;
        }

        private static void ApplyProviderKey(ProcessStartInfo start)
        {
            var data = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "HiveMind");
            var keyEnv = ResolveApiType(data) == "tokenlabs"
                ? "HIVEROUTER_API_KEY"
                : "DEEPSEEK_API_KEY";
            var key = LoadProviderKey(data, keyEnv);
            if (!string.IsNullOrWhiteSpace(key))
            {
                start.EnvironmentVariables[keyEnv] = key;
            }
        }

        private static string ResolveApiType(string data)
        {
            var configured = Environment.GetEnvironmentVariable("HIVEROUTER_API_TYPE");
            if (!string.IsNullOrWhiteSpace(configured))
            {
                return configured.Equals("tokenlabs", StringComparison.OrdinalIgnoreCase)
                    || configured.Equals("tokenlab", StringComparison.OrdinalIgnoreCase)
                    ? "tokenlabs"
                    : "deepseek";
            }
            try
            {
                var profile = Path.Combine(data, "provider.json");
                if (File.Exists(profile))
                {
                    var text = File.ReadAllText(profile);
                    var marker = "\"api_type\"";
                    var index = text.IndexOf(marker, StringComparison.OrdinalIgnoreCase);
                    if (index >= 0)
                    {
                        var colon = text.IndexOf(':', index + marker.Length);
                        var first = colon < 0 ? -1 : text.IndexOf('"', colon + 1);
                        var second = first < 0 ? -1 : text.IndexOf('"', first + 1);
                        if (first >= 0 && second > first)
                        {
                            var value = text.Substring(first + 1, second - first - 1).Trim();
                            return value.Equals("tokenlabs", StringComparison.OrdinalIgnoreCase)
                                || value.Equals("tokenlab", StringComparison.OrdinalIgnoreCase)
                                ? "tokenlabs"
                                : "deepseek";
                        }
                    }
                    if (text.IndexOf("\"tokenlab\"", StringComparison.OrdinalIgnoreCase) >= 0)
                    {
                        return "tokenlabs";
                    }
                }
            }
            catch
            {
                // A damaged profile falls back to the documented DeepSeek default.
            }
            return "deepseek";
        }

        private static string LoadProviderKey(string data, string keyEnv)
        {
            try
            {
                var file = Path.Combine(data, "provider.key");
                if (File.Exists(file))
                {
                    var encrypted = File.ReadAllBytes(file);
                    var plain = ProtectedData.Unprotect(
                        encrypted,
                        Encoding.UTF8.GetBytes(KeyEntropy),
                        DataProtectionScope.CurrentUser);
                    return Encoding.UTF8.GetString(plain).Trim();
                }
            }
            catch
            {
                // Fall through to the environment fallback; never print key material.
            }
            return Environment.GetEnvironmentVariable(keyEnv) ?? string.Empty;
        }

        private static string ResolveWorkspaceRoot()
        {
            string configured = Environment.GetEnvironmentVariable("HIVEMIND_WORKSPACE_ROOT");
            if (!string.IsNullOrWhiteSpace(configured) && Directory.Exists(configured))
            {
                return configured;
            }
            // Installed data and agent workspace stay per-user; never point agents at
            // the whole Documents folder or at a developer checkout.
            string data = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "HiveMind");
            string workspace = Path.Combine(data, "workspace");
            Directory.CreateDirectory(workspace);
            return workspace;
        }

        private static string ResolveServer()
        {            string besideLauncher = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "hiverouter.exe");
            if (File.Exists(besideLauncher))
            {
                return besideLauncher;
            }

            string root = Environment.GetEnvironmentVariable("HIVEMIND_ROOT");
            if (!string.IsNullOrWhiteSpace(root) && Directory.Exists(root))
            {
                string packaged = Path.Combine(root, "hiverouter.exe");
                if (File.Exists(packaged))
                {
                    return packaged;
                }
            }
            string cwd = Directory.GetCurrentDirectory();
            string target = Path.Combine(cwd, @"target\release\hiverouter.exe");
            return File.Exists(target) ? target : null;
        }

        /// <summary>Reuses a healthy HiveMind or finds a free loopback port.</summary>
        private static int FindUsablePort()
        {
            for (int port = FirstPort; port <= LastPort; port++)
            {
                if (!PortOpen(port))
                {
                    return port;
                }
                if (HealthOk(port))
                {
                    return port;
                }
            }
            return 0;
        }

        private static bool PortOpen(int port)
        {
            try
            {
                using (var client = new TcpClient())
                {
                    client.Connect(Host, port);
                    return true;
                }
            }
            catch
            {
                return false;
            }
        }

        private static bool HealthOk(int port)
        {
            try
            {
                using (var client = new WebClient())
                {
                    client.DownloadString("http://" + Host + ":" + port + "/health");
                    return true;
                }
            }
            catch
            {
                return false;
            }
        }

        private static bool WaitForHealth(int port, int timeoutMs)
        {
            var until = DateTime.UtcNow.AddMilliseconds(timeoutMs);
            while (DateTime.UtcNow < until)
            {
                if (HealthOk(port))
                {
                    return true;
                }
                Thread.Sleep(300);
            }
            return false;
        }
    }
}
