import argparse
from pathlib import Path
import shutil
import tkinter as tk
from ppp.instance import Instance
from ppp.solution import Solution
from ppp.gui.main_gui import Application

def main(instance: Instance, solution: Solution, output: Path):
    if not output.exists():
        with output.open("w") as f:
            solution.serialize(f)

    app = Application(instance, solution, output)
    app.mainloop()



def start(args):
    input = args.input.split("/")
    input_file = "inputs/" + input[0] + "/" + input[1].zfill(3) + ".in"
    solution_file = "outputs/" + input[0] + "/" + input[1].zfill(3) + ".out"
    output_file = "edited/" + input[0] + "/" + input[1].zfill(3) + ".out"

    with Path(input_file).open("r") as f:
        instance = Instance.parse(f.readlines())
        solution = Solution(towers=[], instance=instance)

        # If solution file exists, use it
        if Path(solution_file).exists():
            with Path(solution_file).open("r") as f:
                solution = Solution.parse(f, instance)

    output = Path(output_file)
    output.parent.mkdir(parents=True, exist_ok=True)
        
    main(instance, solution, output)

def cli():
    parser = argparse.ArgumentParser(description="Open a problem instance and solution")
    parser.add_argument("input", type=str, help="The input instance file to "
                        "read an instance from in the form <size>/<id>.")
    start(parser.parse_args())
