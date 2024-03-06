# A small script for running patches and quick killing the game.
# Reads from .env file.

from glob import glob
import psutil
import subprocess
import tkinter as tk
from tkinter import ttk
from pathlib import Path

class ExePath:
    def __init__(self, path):
        self.exe_path = path
        self.patch_ver = path.parent.parent.name
        self.cwd = path.parent

def kill_process_by_name(process_name):
    for process in psutil.process_iter(attrs=['pid', 'name']):
        try:
            if process.name().lower() == process_name.lower():
                p = psutil.Process(process.pid)
                p.terminate()
                print(f"Terminated process {process_name} (PID {process.pid})")
        except Exception as e:
            print(e)
            pass

if __name__ == '__main__':
    with open(Path(__file__).parent.parent / '.env') as fp:
        line = fp.read().splitlines()[0]
        patches_path = line.strip().split('=')[1].replace('"', '') + '*'
    print(patches_path)
    dirs = glob(patches_path)
    exes = list(map(lambda x: ExePath(Path(x) / 'Game/eldenring.exe'), dirs))

    root = tk.Tk()
    root.title('ER PT Dev')

    frame = tk.Frame(root, relief=tk.RAISED, borderwidth=1, width=240).pack()

    combo_val = tk.StringVar()
    combo = ttk.Combobox(frame, textvariable=combo_val)
    combo['values'] = [e.patch_ver for e in exes]
    combo['state'] = 'readonly'
    combo.pack(fill=tk.X, padx=5, pady=5)

    def run():
        i = combo.current()
        if i != -1:
            e = exes[i]
            subprocess.Popen(e.exe_path, cwd=e.cwd)

    def kill():
        kill_process_by_name('eldenring.exe')

    btn1 = tk.Button(frame, text='Run', command=run).pack(fill=tk.X, padx=5, pady=5)
    btn2 = tk.Button(frame, text='Kill', command=kill).pack(fill=tk.X, padx=5, pady=5)

    root.mainloop()
