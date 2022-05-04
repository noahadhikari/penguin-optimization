from pathlib import Path
import tkinter as tk
from tkinter import ALL, EventType

from ppp.instance import Instance
from ppp.point import Point
from ppp.solution import Solution


class InfoFrame(tk.Frame):
    def __init__(self, parent, *args, **kwargs):
        tk.Frame.__init__(self, parent, *args, **kwargs)

        
        self.lbl_valid = tk.Label(self)
        self.lbl_valid.pack(padx=10, pady=5, side=tk.RIGHT)

        self.lbl_cost = tk.Label(self)
        self.lbl_cost.pack(padx=10, pady=5, side=tk.RIGHT)

        self.btn_save = tk.Button(self, text="Save", command=parent.save)
        self.btn_save.pack(padx=10, pady=5, side=tk.LEFT)

        self.btn_delete = tk.Button(self, text="Delete", state=tk.DISABLED, command=parent.remove_tower)
        self.btn_delete.pack(padx=10, pady=5, side=tk.RIGHT)

        self.btn_add = tk.Button(self, text="Add", command=parent.bind_add_tower)
        self.btn_add.pack(padx=10, pady=5, side=tk.RIGHT)

        self.lbl_adding = tk.Label(self, text="ADDING TOWER (click to add)")
        self.lbl_selected = tk.Label(self)
    
    def set_valid(self, valid: bool):
        if valid:
            self.lbl_valid.config(
                text="Valid",
                foreground="green",
            )
        else:
            self.lbl_valid.config(
                text="Not valid",
                foreground="red",
            )
    
    def set_cost(self, cost: float):
        self.lbl_cost.config(
            text=f"Cost: {cost:.2f}",
        )

    def set_selected_tower(self, tower: Point | None):
        if tower is None:
            self.lbl_selected.pack_forget()
            self.btn_delete.config(state=tk.DISABLED)
        else:
            self.lbl_selected.config(text="SELECTED TOWER")
            self.lbl_selected.pack(padx=10, pady=5, side=tk.RIGHT)
            self.btn_delete.config(state=tk.NORMAL)

    def set_adding_tower(self, on: bool):
        if on:
            self.lbl_adding.pack(padx=10, pady=5, side=tk.RIGHT)
            self.btn_add.config(state=tk.DISABLED)
        else:
            self.lbl_adding.pack_forget()
            self.btn_add.config(state=tk.NORMAL)
        



class MainCanvas(tk.Canvas):
    def __init__(self, parent, instance: Instance, solution: Solution, canvas_dim, *args, **kwargs):
        super().__init__(parent, height=canvas_dim, width=canvas_dim, *args, **kwargs)

        self.parent = parent
        self.canvas_dim = canvas_dim
        self.grid_dim = instance.grid_side_length
        self.step = self.canvas_dim // self.grid_dim

        self.instance = instance

        self.towers = dict((i,j) for i,j in enumerate(solution.towers))  

        self.selected_tower_id: int | None = None
        self.adding_tower = False

        self.init_grid()
        self.draw_towers()

    def handle_btn_1(self, event):
        self.scan_mark(event.x, event.y)
        # Select tower if it is clicked
        clicked_ids = self.find_withtag(self.gettags("current")[0])

        for id in clicked_ids:
            tower_tags = list(filter(lambda x: "tower" in x, self.gettags(id)))
            if tower_tags:
                self.itemconfig("tower", outline="blue")
                self.selected_tower_id = int(tower_tags[0].split("_")[1])
                self.parent.set_selected_tower(self.towers[self.selected_tower_id])
                self.itemconfig(id, outline="red")
                break

    def move_tower(self, id, dx, dy):
        curr_tower = self.towers[id]
        new_tower = Point(curr_tower.x + dx, curr_tower.y + dy)

        print(new_tower)

        # Assert move is valid
        if (new_tower.x < 0 or new_tower.x >= self.grid_dim) or (new_tower.y < 0 or new_tower.y >= self.grid_dim):
            return

        self.towers[id] = new_tower
        self.parent.update_solution(self.towers.values())

        self.move(f"tower_{id}", dx * self.step, dy * self.step)
        self.move(f"move_with_{id}", dx * self.step, dy * self.step)


    # Add tower on mouse click
    def add_tower(self, event):
        # Snap tower to closest grid point
        x = round(event.x / self.step)
        y = round(event.y / self.step)

        # Assert tower is not already on grid and assert it is valid
        if (x, y) in self.towers.values():
            return
        
        if (x < 0 or x >= self.grid_dim) or (y < 0 or y >= self.grid_dim):
            return

        # Add
        id, tower = len(self.towers), Point(x, y)
        self.towers[id] = tower
        
        # Redraw all towers
        self.draw_towers()

        self.parent.update_solution(self.towers.values())
        self.parent.unbind_add_tower()
    
    # Remove selected tower
    def remove_tower(self):
        if self.selected_tower_id is not None:
            self.delete(f"tower_{self.selected_tower_id}")
            self.delete(f"move_with_{self.selected_tower_id}")
            del self.towers[self.selected_tower_id]
            self.parent.update_solution(self.towers.values())
            self.selected_tower_id = None
            self.parent.set_selected_tower(None)
    
    # Move towers!!
    def move_selected_tower_up(self, event):
        if self.selected_tower_id is not None:
            self.move_tower(self.selected_tower_id, 0, -1)
    
    def move_selected_tower_down(self, event):
        if self.selected_tower_id is not None:
            self.move_tower(self.selected_tower_id, 0, 1)
    
    def move_selected_tower_left(self, event):
        if self.selected_tower_id is not None:
            self.move_tower(self.selected_tower_id, -1, 0)
    
    def move_selected_tower_right(self, event):
        if self.selected_tower_id is not None:
            self.move_tower(self.selected_tower_id, 1, 0)

    def init_grid(self):
        for x in range(self.grid_dim):
            self.create_line(x * self.step, 0, x * self.step, (self.grid_dim - 1) * self.step)
            self.create_line(0, x * self.step, (self.grid_dim - 1) * self.step, x * self.step)
        
        # Draw cities
        for city in self.instance.cities:
            self.create_oval(
                city.x * self.step - self.step // 4,
                city.y * self.step - self.step // 4,
                city.x * self.step + self.step // 4,
                city.y * self.step + self.step // 4,
                fill="red",
            )
    
    def draw_towers(self):
        # Draw penalty circle
        for id, tower in self.towers.items():
            R_p = self.instance.penalty_radius
            self.create_oval(
                tower.x * self.step - R_p * self.step,
                tower.y * self.step - R_p * self.step,
                tower.x * self.step + R_p * self.step,
                tower.y * self.step + R_p * self.step,
                outline="red",
                width=2,
                fill="red",
                stipple="gray50",
                tags=(f"move_with_{id}"),
            )

        # Draw service radius
        for id, tower in self.towers.items():
            R_s = self.instance.coverage_radius
            self.create_oval(
                tower.x * self.step - R_s * self.step,
                tower.y * self.step - R_s * self.step,
                tower.x * self.step + R_s * self.step,
                tower.y * self.step + R_s * self.step,
                outline="green",
                width=2,
                fill="green",
                stipple="gray50",
                tags=(f"move_with_{id}"),
            )

        # Draw tower
        for id, tower in self.towers.items():
            self.create_oval(
                tower.x * self.step - self.step // 4,
                tower.y * self.step - self.step // 4,
                tower.x * self.step + self.step // 4,
                tower.y * self.step + self.step // 4,
                outline="blue",
                width=2,
                fill="blue",
                stipple="gray50",
                tags=(f"tower_{id}", "tower"),
            )
    
    def do_zoom(self, event):
        x = self.canvasx(event.x)
        y = self.canvasy(event.y)
        if event.delta:
            factor = 1.001 ** event.delta
        elif event.num == 5:
            factor = 1.001 ** -10
        elif event.num == 4:
            factor = 1.001 ** 10
        
        self.step = self.step * factor

        self.scale(ALL, x, y, factor, factor)


class Application(tk.Tk):
    "Create top-level Tkinter widget containing all other widgets."

    def __init__(self, instance: Instance, solution: Solution, output: Path):
        tk.Tk.__init__(self)

        self.dim = 720
        self.output = output
        self.solution = solution
        self.add_binding = None

        self.wm_title('PPP')

        self.info_frame = InfoFrame(self, bg="grey") 
        self.info_frame.pack(
            side="top", 
            fill=tk.X, 
        )

        self.info_frame.update()
        # Change geometry such that the info frame is not covered by the canvas
        self.wm_geometry('{}x{}'.format(self.dim, self.dim + self.info_frame.winfo_height()))

        self.canvas = MainCanvas(self, instance, solution, self.dim, bg="white")
        self.canvas.pack(side='top', fill=tk.Y)
        self.canvas.focus_set()
        self._init_zoom()
        self._init_move_towers()


        # Set defaults
        self.info_frame.set_valid(solution.valid())
        self.info_frame.set_cost(solution.penalty())

    def bind_add_tower(self):
        self.info_frame.set_adding_tower(True)
        self.add_binding = self.canvas.bind("<Button-1>", self.canvas.add_tower)

    def unbind_add_tower(self):
        self.info_frame.set_adding_tower(False)
        self.canvas.unbind("<Button-1>", self.add_binding)
        self.canvas.bind('<ButtonPress-1>', self.canvas.handle_btn_1)

    
    def remove_tower(self):
        self.canvas.remove_tower()

    def update_solution(self, towers):
        self.solution = Solution(
            instance=self.solution.instance, 
            towers=towers
        )
        self.update_info_frame(self.solution)

    def set_selected_tower(self, tower):
        self.info_frame.set_selected_tower(tower)


    def save(self):
        with self.output.open("w") as f:
            self.solution.serialize(f)


    def _init_zoom(self):
        # Windows
        self.canvas.bind("<MouseWheel>", self.canvas.do_zoom)
        # Linux
        self.canvas.bind("<Button-4>", self.canvas.do_zoom)
        self.canvas.bind("<Button-5>", self.canvas.do_zoom)

        self.canvas.bind('<ButtonPress-1>', self.canvas.handle_btn_1)
        self.canvas.bind("<B1-Motion>", lambda event: self.canvas.scan_dragto(event.x, event.y, gain=1))
    
    def update_info_frame(self, solution: Solution):
        self.info_frame.set_valid(solution.valid())
        self.info_frame.set_cost(solution.penalty())

    def _init_move_towers(self):
        self.canvas.bind("<Up>", self.canvas.move_selected_tower_up)
        self.canvas.bind("<Down>", self.canvas.move_selected_tower_down)
        self.canvas.bind("<Left>", self.canvas.move_selected_tower_left)
        self.canvas.bind("<Right>", self.canvas.move_selected_tower_right)