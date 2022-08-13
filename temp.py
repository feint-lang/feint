def f(x):
    def g():
        return x, y
    y = "y"
    return g
h = f("hx")
i = f("ix")
