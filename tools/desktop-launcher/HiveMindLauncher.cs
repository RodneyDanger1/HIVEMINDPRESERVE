using System;
using System.Diagnostics;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
using System.Windows.Forms;

namespace HiveMindLauncher
{
    internal static class Program
    {
        private const string Host = "127.0.0.1";
        /// <summary>One launcher at a time; a second launch focuses, never duplicates.</summary>
        private static readonly Mutex InstanceMutex = new Mutex(false, "Local\\HiveMindLauncher.v1");
        private static readonly object LogLock = new object();

        /// <summary>Append one runtime line; logging must never break startup.</summary>
        private static void AppendLog(string path, string line)
        {
            if (line == null)
            {
                return;
            }
            try
            {
                lock (LogLock)
                {
                    File.AppendAllText(path,
                        DateTime.UtcNow.ToString("o") + " " + line + Environment.NewLine);
                }
            }
            catch
            {
                // A read-only or full data directory must not prevent the app
                // from starting; the failed log is not silently presented as
                // evidence because no log line exists to read.
            }
        }
        [DllImport("user32.dll")]
        private static extern bool SetForegroundWindow(IntPtr hWnd);
        private const int FirstPort = 8099;
        private const int LastPort = 8199;
        /// <summary>How many free ports to try when a start attempt loses a race.</summary>
        private const int MaxStartAttempts = 3;
        /// <summary>Per-attempt health bound. A healthy boot is measured at 2.7-5.5s.</summary>
        private const int HealthTimeoutMs = 20000;
        private const string Edge = @"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe";
        private const string ChromeX86 = @"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe";
        private const string ChromeX64 = @"C:\Program Files\Google\Chrome\Application\chrome.exe";
        private const string KeyEntropy = "HiveMind.ProviderKey.v1";
        /// <summary>Marker written into the /health body by the HiveMind server.</summary>
        private const string HealthSchemaMarker = "\"schema\":\"hiverouter";
        private const string HealthBootMarker = "\"boot_id\":";

        [STAThread]
        private static void Main()
        {
            bool instanceOwned = false;
            try
            {
                instanceOwned = InstanceMutex.WaitOne(0);
            }
            catch (AbandonedMutexException)
            {
                instanceOwned = true;
            }
            if (!instanceOwned)
            {
                // The first launcher may still be starting the browser; poll
                // briefly so a fast double-click focuses instead of doing nothing.
                FocusExistingWindow(3000);
                return;
            }
            Process serverProcess = null;
            string data = null;
            bool ownsServer = false;
            // The server must never be orphaned or left behind by an error path:
            // every early return below runs this finally.
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

                data = ResolveDataRoot();
                Directory.CreateDirectory(data);
                string launchToken = LoadOrCreateLaunchToken(data);
                string workspaceRoot = ResolveWorkspaceRoot(data);

                int port = FindReusablePort();
                bool reusedExisting = port != 0;
                string failure = null;
                if (port == 0)
                {
                    if (!TryStartServer(server, workspaceRoot, data, launchToken, out serverProcess, out port, out failure))
                    {
                        MessageBox.Show(
                            "HiveMind could not start its local server.\n\n" + (failure ?? "No free loopback port was available between " + FirstPort + " and " + LastPort + "."),
                            "HiveMind",
                            MessageBoxButtons.OK,
                            MessageBoxIcon.Error);
                        return;
                    }
                    ownsServer = true;
                }

                // Only an owned server has a runtime record this launcher may
                // rewrite/delete. Reused servers keep the record of their owner.
                if (ownsServer)
                {
                    WriteRuntimeFile(data, port, serverProcess);
                }
                // A healthy server plus an existing HiveMind app window means a
                // relaunch should focus that window, never open another one.
                if (reusedExisting && FocusExistingWindow(2000))
                {
                    ownsServer = false;
                    return;
                }
                bool trackableWindow;
                Process window = LaunchWindow(port, launchToken, data, out trackableWindow, out failure);
                if (window == null && !trackableWindow)
                {
                    MessageBox.Show(
                        "HiveMind's server is running, but no browser window could be opened.\n\n" + failure,
                        "HiveMind",
                        MessageBoxButtons.OK,
                        MessageBoxIcon.Error);
                    // The server stays up: the user can open the printed URL later,
                    // and the next launch reuses the healthy instance.
                    ownsServer = false;
                    return;
                }
                if (window == null)
                {
                    // The URL was handed to the default browser, which we cannot wait on.
                    // Keep the server alive and let the launcher exit; the next launch
                    // reuses it through the health probe.
                    ownsServer = false;
                    return;
                }

                var openedAt = DateTime.UtcNow;
                window.WaitForExit();
                // If Edge handed the URL to an already-running HiveMind window,
                // its new process exits immediately; in that case the existing
                // window still owns the server and it must not be killed.
                var lifetime = DateTime.UtcNow - openedAt;
                if (ShouldKillOwnedServer(ownsServer, serverProcess, lifetime))
                {
                    Kill(serverProcess);
                }
                ownsServer = false;
            }
            catch (Exception error)
            {
                MessageBox.Show(
                    "HiveMind could not start:\n\n" + error.Message,
                    "HiveMind",
                    MessageBoxButtons.OK,
                    MessageBoxIcon.Error);
            }
            finally
            {
                if (ownsServer)
                {
                    Kill(serverProcess);
                }
                if (data != null && ownsServer)
                {
                    RemoveRuntimeFile(data);
                }
                try
                {
                    InstanceMutex.ReleaseMutex();
                }
                catch
                {
                    // Process teardown releases it anyway.
                }
            }
        }

        /// <summary>
        /// Focus the single existing HiveMind app window. Returns false when no
        /// such window exists so the caller opens exactly one.
        /// </summary>
        private static bool FocusExistingWindow(int timeoutMs = 0)
        {
            var until = DateTime.UtcNow.AddMilliseconds(timeoutMs);
            do
            {
                try
                {
                    foreach (var name in new[] { "msedge", "chrome" })
                    {
                        foreach (var process in Process.GetProcessesByName(name))
                        {
                            using (process)
                            {
                                if (process.MainWindowHandle == IntPtr.Zero || string.IsNullOrEmpty(process.MainWindowTitle))
                                {
                                    continue;
                                }
                                if (process.MainWindowTitle.IndexOf("HiveMind", StringComparison.OrdinalIgnoreCase) >= 0)
                                {
                                    // Foreground-lock may refuse the raise; still report the
                                    // existing window so we do not open a duplicate.
                                    SetForegroundWindow(process.MainWindowHandle);
                                    return true;
                                }
                            }
                        }
                    }
                }
                catch
                {
                    // Fall through and retry until the deadline.
                }
                if (DateTime.UtcNow >= until) return false;
                Thread.Sleep(150);
            } while (true);
        }

        private static bool ShouldKillOwnedServer(bool ownsServer, Process serverProcess, TimeSpan lifetime)
        {
            return ownsServer
                && serverProcess != null
                && lifetime.TotalSeconds >= 3
                && !serverProcess.HasExited;
        }

        private static void Kill(Process process)
        {
            try
            {
                if (process != null && !process.HasExited)
                {
                    process.Kill();
                }
            }
            catch
            {
                // A process that already exited is the expected outcome here.
            }
        }

        /// <summary>
        /// Data root: <c>install.json</c> beside the launcher (written by
        /// install-desktop.ps1) wins, so a custom -DataRoot is honoured instead of
        /// being silently replaced by the per-user default.
        /// </summary>
        private static string ResolveDataRoot()
        {
            string configured = ReadInstallValue("data_root");
            if (!string.IsNullOrWhiteSpace(configured))
            {
                return configured;
            }
            return Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "HiveMind");
        }

        /// <summary>
        /// Workspace root: explicit env override, then install.json, then
        /// &lt;data&gt;\workspace. Installed data and the agent workspace stay per-user;
        /// never point agents at the whole Documents folder or a developer checkout.
        /// </summary>
        private static string ResolveWorkspaceRoot(string data)
        {
            string configured = Environment.GetEnvironmentVariable("HIVEMIND_WORKSPACE_ROOT");
            if (!string.IsNullOrWhiteSpace(configured) && Directory.Exists(configured))
            {
                return configured;
            }
            string installed = ReadInstallValue("workspace_root");
            if (!string.IsNullOrWhiteSpace(installed))
            {
                Directory.CreateDirectory(installed);
                return installed;
            }
            string workspace = Path.Combine(data, "workspace");
            Directory.CreateDirectory(workspace);
            return workspace;
        }

        private static string ReadInstallValue(string key)
        {
            try
            {
                var file = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "install.json");
                if (!File.Exists(file))
                {
                    return null;
                }
                return ReadJsonString(File.ReadAllText(file), key);
            }
            catch
            {
                // A damaged install record falls back to the documented default.
                return null;
            }
        }

        /// <summary>
        /// Reads one string value out of a small flat JSON object (install.json /
        /// provider.json) without a JSON dependency, honouring JSON escapes.
        /// </summary>
        private static string ReadJsonString(string text, string key)
        {
            if (string.IsNullOrEmpty(text) || string.IsNullOrEmpty(key))
            {
                return null;
            }
            string marker = "\"" + key + "\"";
            int index = text.IndexOf(marker, StringComparison.OrdinalIgnoreCase);
            if (index < 0)
            {
                return null;
            }
            int colon = text.IndexOf(':', index + marker.Length);
            if (colon < 0)
            {
                return null;
            }
            int cursor = colon + 1;
            while (cursor < text.Length && char.IsWhiteSpace(text[cursor]))
            {
                cursor++;
            }
            if (cursor >= text.Length || text[cursor] != '"')
            {
                return null;
            }
            var value = new StringBuilder();
            for (int i = cursor + 1; i < text.Length; i++)
            {
                char current = text[i];
                if (current == '\\' && i + 1 < text.Length)
                {
                    char escape = text[++i];
                    switch (escape)
                    {
                        case 'n': value.Append('\n'); break;
                        case 'r': value.Append('\r'); break;
                        case 't': value.Append('\t'); break;
                        case 'b': value.Append('\b'); break;
                        case 'f': value.Append('\f'); break;
                        default: value.Append(escape); break;
                    }
                    continue;
                }
                if (current == '"')
                {
                    string parsed = value.ToString();
                    return parsed.Length == 0 ? null : parsed;
                }
                value.Append(current);
            }
            return null;
        }

        /// <summary>
        /// Loads or creates the per-launch secret that gates browser reads. The file
        /// lives in the per-user data directory and is never logged or written into
        /// the package folder.
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

        /// <summary>
        /// Records the live port so the installer and diagnostics do not have to
        /// guess which port in 8099..8199 the launcher chose.
        /// </summary>
        private static void WriteRuntimeFile(string data, int port, Process serverProcess)
        {
            try
            {
                var runtime = new StringBuilder();
                runtime.Append("{\n");
                runtime.Append("  \"port\": ").Append(port).Append(",\n");
                runtime.Append("  \"pid\": ")
                    .Append(serverProcess != null && !serverProcess.HasExited ? serverProcess.Id : 0)
                    .Append(",\n");
                runtime.Append("  \"started_at\": \"").Append(DateTime.UtcNow.ToString("o")).Append("\",\n");
                runtime.Append("  \"data_root\": \"").Append(JsonEscape(data)).Append("\"\n");
                runtime.Append("}\n");
                File.WriteAllText(Path.Combine(data, "runtime.json"), runtime.ToString(), new UTF8Encoding(false));
            }
            catch
            {
                // The runtime record is a convenience; a locked data dir must not
                // stop the app from starting.
            }
        }

        private static void RemoveRuntimeFile(string data)
        {
            try
            {
                var file = Path.Combine(data, "runtime.json");
                if (File.Exists(file))
                {
                    File.Delete(file);
                }
            }
            catch
            {
                // Stale runtime records are detected by probing the recorded port.
            }
        }

        private static string JsonEscape(string value)
        {
            return value.Replace("\\", "\\\\").Replace("\"", "\\\"");
        }

        /// <summary>
        /// Loads the provider key from a DPAPI-encrypted per-user file and
        /// injects it into the server child's environment. The key is never
        /// written to source, a command line, a log, or stdout/stderr.
        /// </summary>
        private static void ApplyProviderKey(ProcessStartInfo start, string data)
        {
            string apiType = ResolveApiType(data);
            string keyEnv = apiType == "tokenlabs" ? "HIVEROUTER_API_KEY" : "DEEPSEEK_API_KEY";
            var key = LoadProviderKey(data, keyEnv);
            if (!string.IsNullOrWhiteSpace(key))
            {
                start.EnvironmentVariables[keyEnv] = key;
            }
            // The server resolves the profile from LOCALAPPDATA by default; point it
            // at the same file this launcher read so a custom data root stays coherent.
            start.EnvironmentVariables["HIVEROUTER_PROVIDER_FILE"] = Path.Combine(data, "provider.json");
        }

        private static string ResolveApiType(string data)
        {
            var configured = Environment.GetEnvironmentVariable("HIVEROUTER_API_TYPE");
            if (!string.IsNullOrWhiteSpace(configured))
            {
                return NormalizeApiType(configured);
            }
            try
            {
                var profile = Path.Combine(data, "provider.json");
                if (File.Exists(profile))
                {
                    string text = File.ReadAllText(profile);
                    string apiType = ReadJsonString(text, "api_type");
                    if (!string.IsNullOrWhiteSpace(apiType))
                    {
                        return NormalizeApiType(apiType);
                    }
                    string name = ReadJsonString(text, "name");
                    if (!string.IsNullOrWhiteSpace(name))
                    {
                        return NormalizeApiType(name);
                    }
                }
            }
            catch
            {
                // A damaged profile falls back to the documented DeepSeek default.
            }
            return "deepseek";
        }

        private static string NormalizeApiType(string value)
        {
            return value.Equals("tokenlabs", StringComparison.OrdinalIgnoreCase)
                || value.Equals("tokenlab", StringComparison.OrdinalIgnoreCase)
                || value.Equals("custom", StringComparison.OrdinalIgnoreCase)
                || value.Equals("external", StringComparison.OrdinalIgnoreCase)
                ? "tokenlabs"
                : "deepseek";
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

        private static string ResolveServer()
        {
            string besideLauncher = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "hiverouter.exe");
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

        /// <summary>Reuses a healthy HiveMind server already listening in range.</summary>
        private static int FindReusablePort()
        {
            for (int port = FirstPort; port <= LastPort; port++)
            {
                if (PortOpen(port) && HealthIsHiveMind(port))
                {
                    return port;
                }
            }
            return 0;
        }

        /// <summary>
        /// Starts the server on the first free loopback port. A start attempt that
        /// loses a race (another process grabbed the port, or a concurrent launcher
        /// won) is retried on the next free port instead of being reported as a
        /// 20-second "did not become ready" timeout.
        /// </summary>
        private static bool TryStartServer(
            string server,
            string workspaceRoot,
            string data,
            string launchToken,
            out Process serverProcess,
            out int port,
            out string failure)
        {
            serverProcess = null;
            port = 0;
            failure = null;
            int attempts = 0;
            for (int candidate = FirstPort; candidate <= LastPort; candidate++)
            {
                if (PortOpen(candidate))
                {
                    continue;
                }
                if (attempts >= MaxStartAttempts)
                {
                    failure = "Gave up after " + attempts + " start attempts; the last error was: " + (failure ?? "unknown");
                    return false;
                }
                attempts++;
                if (TryStartOn(server, workspaceRoot, data, launchToken, candidate, out serverProcess, out failure))
                {
                    port = candidate;
                    return true;
                }
            }
            if (failure == null)
            {
                failure = "No free loopback port was found between " + FirstPort + " and " + LastPort + ".";
            }
            return false;
        }

        private static bool TryStartOn(
            string server,
            string workspaceRoot,
            string data,
            string launchToken,
            int candidate,
            out Process serverProcess,
            out string failure)
        {
            serverProcess = null;
            failure = null;
            ProcessStartInfo start;
            try
            {
                start = new ProcessStartInfo
                {
                    FileName = server,
                    Arguments = "--bind " + Host + ":" + candidate + " --data-dir \"" + data + "\" --workspace-root \"" + workspaceRoot + "\"",
                    WorkingDirectory = workspaceRoot,
                    UseShellExecute = false,
                    CreateNoWindow = true,
                    // Redirect only with the asynchronous readers below; a chatty
                    // child would otherwise fill a pipe and deadlock before health.
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                };
                ApplyProviderKey(start, data);
                start.EnvironmentVariables["HIVEROUTER_LAUNCH_TOKEN"] = launchToken;
                serverProcess = Process.Start(start);
            }
            catch (Exception error)
            {
                failure = "The server process could not be started: " + error.Message;
                serverProcess = null;
                return false;
            }
            if (serverProcess == null)
            {
                failure = "The server process did not start.";
                return false;
            }
            string stdoutLog = Path.Combine(data, "server.stdout.log");
            string stderrLog = Path.Combine(data, "server.stderr.log");
            serverProcess.OutputDataReceived += (_, args) => AppendLog(stdoutLog, args.Data);
            serverProcess.ErrorDataReceived += (_, args) => AppendLog(stderrLog, args.Data);
            serverProcess.BeginOutputReadLine();
            serverProcess.BeginErrorReadLine();
            AppendLog(Path.Combine(data, "launcher.log"),
                "server starting pid=" + serverProcess.Id + " port=" + candidate);
            if (WaitForHealth(candidate, HealthTimeoutMs, serverProcess))
            {
                return true;
            }
            bool exited = serverProcess.HasExited;
            int exitCode = 0;
            if (exited)
            {
                try
                {
                    exitCode = serverProcess.ExitCode;
                }
                catch
                {
                    exitCode = 0;
                }
            }
            Kill(serverProcess);
            failure = exited
                ? "hiverouter.exe exited with code " + exitCode + " before it became healthy on port " + candidate
                    + ". A port conflict or an unwritable data directory (" + data + ") is the usual cause."
                : "hiverouter.exe did not answer /health on port " + candidate + " within "
                    + (HealthTimeoutMs / 1000) + " seconds.";
            serverProcess = null;
            return false;
        }

        private static bool PortOpen(int port)
        {
            try
            {
                using (var client = new TcpClient())
                {
                    var connect = client.BeginConnect(Host, port, null, null);
                    if (!connect.AsyncWaitHandle.WaitOne(TimeSpan.FromMilliseconds(300)))
                    {
                        return false;
                    }
                    client.EndConnect(connect);
                    return true;
                }
            }
            catch
            {
                return false;
            }
        }

        /// <summary>
        /// True only for this product's /health: another local app listening in the
        /// port range must not be mistaken for a HiveMind server.
        /// </summary>
        private static bool HealthIsHiveMind(int port)
        {
            string body = ReadHealthBody(port);
            return body != null && HealthBodyIsHiveMind(body);
        }

        private static bool HealthBodyIsHiveMind(string body)
        {
            if (string.IsNullOrEmpty(body))
            {
                return false;
            }
            string compact = body.Replace(" ", string.Empty)
                .Replace("\r", string.Empty)
                .Replace("\n", string.Empty)
                .Replace("\t", string.Empty);
            return compact.IndexOf(HealthSchemaMarker, StringComparison.Ordinal) >= 0
                && compact.IndexOf(HealthBootMarker, StringComparison.Ordinal) >= 0;
        }

        private static string ReadHealthBody(int port)
        {
            try
            {
                var request = (HttpWebRequest)WebRequest.Create("http://" + Host + ":" + port + "/health");
                request.Proxy = null;
                request.Timeout = 1500;
                request.ReadWriteTimeout = 1500;
                request.KeepAlive = false;
                using (var response = (HttpWebResponse)request.GetResponse())
                using (var reader = new StreamReader(response.GetResponseStream()))
                {
                    return reader.ReadToEnd();
                }
            }
            catch (WebException error)
            {
                // A degraded server may answer /health with a non-2xx status. The
                // body still identifies the instance; discarding it would start a
                // second server against the same data directory.
                var response = error.Response as HttpWebResponse;
                if (response == null) return null;
                try
                {
                    using (response)
                    using (var reader = new StreamReader(response.GetResponseStream()))
                    {
                        return reader.ReadToEnd();
                    }
                }
                catch
                {
                    return null;
                }
            }
            catch
            {
                return null;
            }
        }

        private static bool WaitForHealth(int port, int timeoutMs, Process serverProcess)
        {
            var until = DateTime.UtcNow.AddMilliseconds(timeoutMs);
            while (DateTime.UtcNow < until)
            {
                if (HealthIsHiveMind(port))
                {
                    return true;
                }
                if (serverProcess != null && serverProcess.HasExited)
                {
                    return false;
                }
                Thread.Sleep(250);
            }
            return false;
        }

        /// <summary>
        /// Opens the UI in an app window. Returns the window process when it can be
        /// tracked; when the URL is handed to the default browser,
        /// <paramref name="trackable"/> is true and the process is null.
        /// </summary>
        private static Process LaunchWindow(
            int port,
            string launchToken,
            string data,
            out bool trackable,
            out string failure)
        {
            trackable = false;
            failure = null;
            string url = "http://" + Host + ":" + port + "/?launch=" + launchToken;
            string profile = Path.Combine(data, "window");
            Directory.CreateDirectory(profile);
            string[] candidates = { Edge, ChromeX86, ChromeX64 };
            foreach (var browserPath in candidates)
            {
                if (!File.Exists(browserPath))
                {
                    continue;
                }
                try
                {
                    var browser = new ProcessStartInfo
                    {
                        FileName = browserPath,
                        Arguments = "--app=" + url
                            + " --window-size=1440,900 --user-data-dir=\"" + profile + "\"",
                        UseShellExecute = false,
                    };
                    Process window = Process.Start(browser);
                    if (window != null)
                    {
                        return window;
                    }
                }
                catch (Exception error)
                {
                    failure = "The browser window could not be opened: " + error.Message;
                }
            }
            try
            {
                Process.Start(new ProcessStartInfo { FileName = url, UseShellExecute = true });
                trackable = true;
                return null;
            }
            catch (Exception error)
            {
                failure = "No Edge or Chrome installation was found and the default browser could not open "
                    + url + " (" + error.Message + "). Open that address manually.";
                return null;
            }
        }
    }
}
