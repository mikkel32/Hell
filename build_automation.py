import os
import sys
import ctypes
import subprocess
import time

# Define the shared log file
LOG_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "build_debug_v2.log")

def is_admin():
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False

def run_privileged_build():
    """
    This function runs ONLY when the script is elevated as Administrator.
    It executes the PowerShell script and ensures output is written to the log file.
    """
    ps_script = os.path.join(os.path.dirname(os.path.abspath(__file__)), "build_sign.ps1")
    
    # Run the PowerShell script
    # Note: build_sign.ps1 handles its own logging to LOG_FILE, but we add a wrapper just in case.
    cmd = ["powershell.exe", "-ExecutionPolicy", "Bypass", "-NoProfile", "-File", ps_script]
    
    try:
        # We run it and wait. The output is handled by the PS1 script's custom Log function.
        subprocess.run(cmd, check=True)
        with open(LOG_FILE, "a") as f:
            f.write("\n>>> BUILD_PROCESS_COMPLETED_SUCCESS\n")
    except subprocess.CalledProcessError:
        with open(LOG_FILE, "a") as f:
            f.write("\n>>> BUILD_PROCESS_COMPLETED_FAILURE\n")
    except Exception as e:
        with open(LOG_FILE, "a") as f:
            f.write(f"\n>>> BUILD_PROCESS_CRASH: {e}\n")
            f.write("\n>>> BUILD_PROCESS_COMPLETED_FAILURE\n")

def main():
    if is_admin():
        # --- ADMIN MODE ---
        # We are the worker process. Run the build.
        run_privileged_build()
        # Keep window open briefly if it was visible, so user knows it finished
        time.sleep(2) 
    else:
        # --- USER MODE ---
        # We are the interface. initialize logs and launch the worker.
        
        # 1. Reset Log File
        # We create it if missing, or truncate if exists.
        with open(LOG_FILE, 'w') as f:
            f.write(f">>> [Launcher] Starting Build Process at {time.ctime()}\n")
            f.write(f">>> [Launcher] Log File: {LOG_FILE}\n")

        # 2. Launch Admin Worker
        print(">>> Requesting Elevation...")
        print(">>> Please accept the UAC prompt to continue.")
        
        # ShellExecuteW params:
        # hwnd=None, lpOperation="runas", lpFile=python_exe, lpParameters="script.py", lpDirectory=None, nShowCmd=SW_SHOWNORMAL(1)
        params = f'"{os.path.abspath(__file__)}"'
        hinstance = ctypes.windll.shell32.ShellExecuteW(
            None, "runas", sys.executable, params, None, 1
        )

        if hinstance <= 32:
            print(f">>> Elevation failed. Error code: {hinstance}")
            sys.exit(1)

        print(">>> Elevation successful. Streaming remote logs to this terminal...")
        print("----------------------------------------------------------------")

        # 3. Tail the Log File
        # Read from the file until we see the completion marker
        try:
            with open(LOG_FILE, 'r') as f:
                # Seek to end initially? No, we want to see the start.
                while True:
                    line = f.readline()
                    if line:
                        print(line, end='', flush=True) # Log lines already have newlines
                        if "BUILD_PROCESS_COMPLETED" in line:
                            break
                    else:
                        time.sleep(0.1) # Wait for new data
        except KeyboardInterrupt:
            print("\n>>> [Launcher] Interrupted by User.")
        except Exception as e:
            print(f"\n>>> [Launcher] Error verifying logs: {e}")

if __name__ == "__main__":
    main()
