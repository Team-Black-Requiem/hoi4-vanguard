import os
import inspect
import argparse

class GFXEntryGenerator:
    def __init__(self):
        self.base_dir = os.getcwd()
        self.spritetype_template = """
            SpriteType = {
                name = "<sprite name>"
                texturefile = "<folder>/<filename>"
            }
        """
        self.shines_template = """
            SpriteType = {
                name = "<sprite name>_shine"
                texturefile = "<folder>/<filename>"
                effectFile = "gfx/FX/buttonstate.lua"
                animation = {
                    animationmaskfile = "gfx/interface/goals/<subfolder>/<filename>"
                    animationtexturefile = "gfx/interface/goals/shine_overlay.dds"
                    animationrotation = -90.0
                    animationlooping = no
                    animationtime = 0.75
                    animationdelay = 0
                    animationblendmode = "add"
                    animationtype = "scrolling"
                    animationrotationoffset = { x = 0.0 y = 0.0 }
                    animationtexturescale = { x = 1.0 y = 1.0 }
                }
    
                animation = {
                    animationmaskfile = "<folder>/<filename>"
                    animationtexturefile = "gfx/interface/goals/shine_overlay.dds"
                    animationrotation = 90.0
                    animationlooping = no
                    animationtime = 0.75
                    animationdelay = 0
                    animationblendmode = "add"
                    animationtype = "scrolling"
                    animationrotationoffset = { x = 0.0 y = 0.0 }
                    animationtexturescale = { x = 1.0 y = 1.0 }
                }
                    legacy_lazy_load = no
            }
        """

    def find_files(self, subfolder, extensions):
        script_dir = os.path.dirname(os.path.abspath(inspect.stack()[-1].filename))
        search_path = os.path.join(self.base_dir, 'gfx', subfolder)
        discovered_files = []
        for root, dirs, files in os.walk(search_path):
            for file in files:
                if any(file.lower().endswith(extension) for extension in extensions):
                    file_name = file
                    folder_path = os.path.relpath(root, script_dir).replace('\\', '/')
                    discovered_files.append((file_name, folder_path))

        return discovered_files

    def generate_entries(self, file_list, output_file, template):
        with open(output_file, 'w') as file:
            file.write("spriteTypes = {\n")
            for file_name, folder_path in file_list:
                sprite_name = 'GFX_' + os.path.splitext(file_name)[0]
                entry = template.replace('<sprite name>', sprite_name)
                entry = entry.replace('<folder>', folder_path)
                entry = entry.replace('<filename>', file_name)
                entry = entry.replace('//', '/')
                file.write(entry)
            file.write("}")

    def generate_gfx_entries(self, args):
        if args.subfolder:
            subfolder_files = self.find_files(args.subfolder, ['.dds', '.png'])
            output_filename = '{}.gfx'.format(os.path.basename(args.subfolder))
            self.generate_entries(subfolder_files, output_filename, self.spritetype_template)
            print('Successfully generated spritetype entries for {} files in the {} subfolder.'.format(len(subfolder_files), args.subfolder))
            if args.subfolder == 'interface/goals':
                self.generate_entries(subfolder_files, 'goals_shines.gfx', self.shines_template)
                print('Successfully generated shines entries for {} files in the {} subfolder.'.format(len(subfolder_files), args.subfolder))
            
        if args.goals_shines:
            goals_files = self.find_files('interface/goals', '.dds')
            self.generate_entries(goals_files, 'goals.gfx', self.spritetype_template)
            self.generate_entries(goals_files, 'goals_shines.gfx', self.shines_template)
            print('Successfully generated goals and shines entries for {} files.'.format(len(goals_files)))

        if args.ideas:
            ideas_files = self.find_files('interface/ideas', '.dds')
            self.generate_entries(ideas_files, 'ideas.gfx', self.spritetype_template)
            print('Successfully generated ideas entries for {} files.'.format(len(ideas_files)))

        if args.event_pictures:
            event_pictures_files = self.find_files('event_pictures', '.dds')
            self.generate_entries(event_pictures_files, 'event_pictures.gfx', self.spritetype_template)
            print('Successfully generated event pictures entries for {} files.'.format(len(event_pictures_files)))
            
        if args.leader_gfx:
            leader_portrait_files = self.find_files('leaders', '.dds')
            self.generate_entries(leader_portrait_files, 'leaders.gfx', self.spritetype_template)
            print('Successfully generated leader portrait entries for {} files.'.format(len(leader_portrait_files)))


def main():
    parser = argparse.ArgumentParser(description='Generate goals, shines, and ideas entries')
    parser.add_argument('--goals-shines', action='store_true',
                        help='generate goals and shine entries')
    parser.add_argument('--ideas', action='store_true',
                        help='generate ideas entries')
    parser.add_argument('--event-pictures', action='store_true',
                        help='generate event pictures entries')
    parser.add_argument('--leader-gfx', action='store_true',
                        help='generate leader portrait entries')
    parser.add_argument('--subfolder', type=str, default='',
                        help='subfolder within /gfx/')
    args = parser.parse_args()

    generator = GFXEntryGenerator()
    generator.generate_gfx_entries(args)


if __name__ == '__main__':
    main()
