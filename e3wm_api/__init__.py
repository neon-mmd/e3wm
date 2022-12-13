class hacker:
    bindings = []

    @staticmethod
    def learn_bindings(key, cmd, group, desc):
        hacker.bindings.append((key, cmd, group, desc))

    @staticmethod
    def change_layout():
        return "change_layout"

    @staticmethod
    def cycle_window_focus_forward():
        return "cycle_window_focus_forward"

    @staticmethod
    def cycle_window_focus_backward():
        return "cycle_window_focus_backward"

    @staticmethod
    def cycle_window_forward():
        return "cycle_window_forward"

    @staticmethod
    def cycle_window_backward():
        return "cycle_window_backward"

    @staticmethod
    def stop():
        return "quit"

    @staticmethod
    def redoit():
        return "reload"
