# Verify I didn't introduce any syntax errors
import subprocess
try:
    subprocess.check_output(['cargo', 'check', '-p', 'idle-daemon'], stderr=subprocess.STDOUT)
    print("Check OK")
except subprocess.CalledProcessError as e:
    print("Check failed:")
    print(e.output.decode())
