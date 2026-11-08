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
        private const int PreferredPort = 8099;
        private const string DefaultRoot = @"C:\Users\Riley\Desktop\hiverouter-v1";
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

                int port = PreferredPort;
                Process serverProcess = null;
                if (PortOpen(port) && !HealthOk(port))
                {
                    MessageBox.Show(
                        "Port " + port + " is in use by another application.\n\n" +
                        "Close that application and start HiveMind again.",
                        "HiveMind",
                        MessageBoxButtons.OK,
                        MessageBoxIcon.Error);
                    return;
                }
                if (!PortOpen(port))
                {
                    var start = new ProcessStartInfo
                    {
                        FileName = server,
                        Arguments = "--bind " + Host + ":" + port + " --data-dir \"" + data + "\"",
                        WorkingDirectory = Path.GetDirectoryName(server),
                        UseShellExecute = false,
                        CreateNoWindow = true,
                        RedirectStandardOutput = true,
                        RedirectStandardError = true,
                    };
                    ApplyProviderKey(start);
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
                        + "/ --window-size=1440,900 --user-data-dir=\"" + profile + "\"",
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
        private static void ApplyProviderKey(ProcessStartInfo start)
        {
            var key = LoadProviderKey();
            if (!string.IsNullOrWhiteSpace(key))
            {
                start.EnvironmentVariables["DEEPSEEK_API_KEY"] = key;
            }
        }

        private static string LoadProviderKey()
        {
            try
            {
                var data = Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                    "HiveMind");
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
            return Environment.GetEnvironmentVariable("DEEPSEEK_API_KEY") ?? string.Empty;
        }

        private static string ResolveServer()
        {            string besideLauncher = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "hiverouter.exe");
            if (File.Exists(besideLauncher))
            {
                return besideLauncher;
            }

            string root = Environment.GetEnvironmentVariable("HIVEMIND_ROOT");
            if (string.IsNullOrWhiteSpace(root) || !Directory.Exists(root))
            {
                root = DefaultRoot;
            }
            string packaged = Path.Combine(root, "hiverouter.exe");
            if (File.Exists(packaged))
            {
                return packaged;
            }
            string target = Path.Combine(root, @"target\release\hiverouter.exe");
            return File.Exists(target) ? target : null;
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
