import os
import inspect
import argparse

focus_template = """
focus = {
    id = <focus_id>
    icon = <icon>
    <prerequisite>
    x = <x>
    y = <y>

    cost = <cost>

    ai_will_do = {
        <ai_will_do>
    }

    completion_reward = {
        <completion_reward>
    }
}
"""



class FocusGenerator:
    def __init__(self):
        self.base_dir = os.getcwd()

    def generate_focus(self, focus_id, icon, prerequisite, x, y, cost, ai_will_do, completion_reward):
        focus_file = f"focus_{focus_id}.txt"
        focus_path = os.path.join(self.base_dir, focus_file)
        self.generate_focus_file(focus_path, focus_id, icon, prerequisite, x, y, cost, ai_will_do, completion_reward)

    def generate_focus_file(self, focus_path, focus_id, icon, prerequisite, x, y, cost, ai_will_do, completion_reward):
        with open(focus_path, 'w') as file:
            file.write(focus_template.replace('<focus_id>', focus_id)
                                    .replace('<icon>', icon)
                                    .replace('<prerequisite>', prerequisite)
                                    .replace('<x>', x)
                                    .replace('<y>', y)
                                    .replace('<cost>', cost)
                                    .replace('<ai_will_do>', ai_will_do)
                                    .replace('<completion_reward>', completion_reward))

        print(f"Successfully generated focus file: {focus_path}")