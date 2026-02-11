import os
import sys
import subprocess
import shutil
import time

def main():
    print(">>> [SENTIENCE] Initiating Build Sequence...")
    
    # 1. Paths
    root_dir = os.path.dirname(os.path.abspath(__file__))
    target_dir = os.path.join(root_dir, "target", "release")
    exe_name = "kawaii_cleaner_pro.exe"
    final_name = "KawaiiCleaner.exe"
    final_path = os.path.join(root_dir, final_name)
    
    # 2. Cargo Build
    print(">>> [BUILD] Compiling Singularity Engine (Release)...")
    cargo_path = r"C:\Users\Bruger\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe"
    try:
        subprocess.run([cargo_path, "build", "--release"], cwd=root_dir, check=True)
    except subprocess.CalledProcessError:
        print(">>> [ERROR] Compilation Failed.")
        sys.exit(1)
        
    # 3. Artifact Handling
    source_path = os.path.join(target_dir, exe_name)
    if os.path.exists(source_path):
        print(f">>> [ARTIFACT] Found binary: {source_path}")
        try:
            shutil.copy2(source_path, final_path)
            print(f">>> [DEPLOY] Copied to: {final_path}")
            print(">>> [SUCCESS] Build Complete. Sentience is active.")
        except Exception as e:
            print(f">>> [ERROR] Copy failed: {e}")
    else:
        print(f">>> [ERROR] Binary not found at {source_path}")
        sys.exit(1)

if __name__ == "__main__":
    main()
