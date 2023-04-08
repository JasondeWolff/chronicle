import subprocess
import sys

shaders = [
    "shader.vert",
    "shader.frag"
]

cwd = ""
if len(sys.argv) > 1:
    cwd = sys.argv[1] + "/"

for shader in shaders:
    cmd_str = "glslc " + cwd + "src/" + shader + " -o " + cwd + "bin/" + shader + ".spv"
    subprocess.run(cmd_str, shell = True)