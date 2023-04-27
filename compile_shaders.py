import subprocess
import sys

shaders = [
    "shader.vert",
    "shader.frag",
    "imgui.vert",
    "imgui.frag",
    "raytracing/raytrace.rgen",
    "raytracing/raytrace.rchit",
    "raytracing/raytrace.rmiss",
    "raytracing/raytrace_shadow.rmiss"
]

cwd = ""
if len(sys.argv) > 1:
    cwd = sys.argv[1] + "/"

for shader in shaders:
    cmd_str = "glslc --target-env=vulkan1.3 -g " + cwd + "src/" + shader + " -o " + cwd + "bin/" + shader + ".spv"
    subprocess.run(cmd_str, shell = True)