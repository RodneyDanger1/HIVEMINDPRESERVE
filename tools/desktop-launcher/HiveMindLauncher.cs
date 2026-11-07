using System;
using System.Diagnostics;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using System.Windows.Forms;

namespace HiveMindLauncher
{
    internal static class Program
    {
        private const string Host = "127.0.0.1";
        private const int Port = 8099;
        private const string DefaultRoot = @"C:\Users\Riley\Desktop\hiverouter-v1";
        private const string Edge = @"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe";

        [STAThread]
        private static void Main()
        {
            try
            {
                string root = Environment.GetEnvironmentVariable("HIVEMIND_ROOT");
                if (string.IsNullOrWhiteSpace(root) || !Directory.Exists(root))
                {
                    root = DefaultRoot;
                }

                string server = Path.Combine(root, @"target\release\hiverouter.exe");
                if (!File.Exists(server))
                {
                    MessageBox.Show(
                        "The HiveMind server is not built yet.\n\nExpected:\n" + server,
                        "HiveMind",
                        MessageBoxButtons.OK,
                        MessageBoxIcon.Error);
                    return;
                }

                string data = Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                    "HiveMind");
                Directory.CreateDirectory(data);

                Process serverProcess = null;
                if (!PortOpen())
                {
                    var start = new ProcessStartInfo
                    {
                        FileName = server,
                        Arguments = "--bind " + Host + ":" + Port + " --data-dir \"" + data + "\"",
                        WorkingDirectory = root,
                        UseShellExecute = false,
                        CreateNoWindow = true,
                        RedirectStandardOutput = true,
                        RedirectStandardError = true,
                    };
                    serverProcess = Process.Start(start);
                    if (!WaitForHealth(60000))
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
                var browser = new ProcessStartInfo
                {
                    FileName = File.Exists(Edge) ? Edge : @"C:\Program Files\Google\Chrome\Application\chrome.exe",
                    Arguments = "--app=http://" + Host + ":" + Port + "/ --window-size=1440,900 --user-data-dir=\"" + profile + "\"",
                    UseShellExecute = false,
                };
                Process window = Process.Start(browser);
                if (window != null)
                {
                    window.WaitForExit();
                }

                if (serverProcess != null && !serverProcess.HasExited)
                {
                    serverProcess.Kill();
                }
            }
            catch (Exception error)
            {
                MessageBox.Show("HiveMind could not start:\n\n" + error.Message, "HiveMind", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private static bool PortOpen()
        {
            try
            {
                using (var client = new TcpClient())
                {
                    client.Connect(Host, Port);
                    return true;
                }
            }
            catch
            {
                return false;
            }
        }

        private static bool WaitForHealth(int timeoutMs)
        {
            var until = DateTime.UtcNow.AddMilliseconds(timeoutMs);
            while (DateTime.UtcNow < until)
            {
                try
                {
                    using (var client = new WebClient())
                    {
                        string body = client.DownloadString("http://" + Host + ":" + Port + "/health");
                        if (body.IndexOf("\"status\"", StringComparison.Ordinal) >= 0)
                        {
                            return true;
                        }
                    }
                }
                catch
                {
                    // The server is still binding; retry until the bounded deadline.
                }
                Thread.Sleep(300);
            }
            return false;
        }
    }
}
