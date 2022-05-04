    window = tk.Tk()


    info = tk.Frame(
        height=100
    )
    info.pack(side="top", fill=tk.X, expand=True)
    visualizer = tk.Frame(
        bg="red",
        width=1000,
        height=100,
    )


    label_b = tk.Label(master=visualizer, text="I'm in Frame B")
    label_b.pack()

    info.pack()
    visualizer.pack()