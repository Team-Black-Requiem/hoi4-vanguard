import os
import re
import string
import inspect
import argparse

class FileFinder:
    def __init__(self):
        self.base_dir = os.getcwd()

    def find_files(self, folder, subfolder, extensions):
        script_dir = os.path.dirname(os.path.abspath(inspect.stack()[-1].filename))
        search_path = os.path.join(self.base_dir, folder, subfolder)
        discovered_files = []
        for root, dirs, files in os.walk(search_path):
            for file in files:
                if any(file.lower().endswith(extension) for extension in extensions):
                    #file_name = file
                    #folder_path = os.path.relpath(root, script_dir).replace('\\', '/')
                    discovered_files.append((file, os.path.relpath(root, script_dir).replace('\\', '/')))

        return discovered_files

class GFXEntryGenerator:
    def __init__(self):
        self.GFXfinder = FileFinder(folder="gfx")
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
                    animationmaskfile = "<folder>/<filename>"
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

    def parse_gfx_file(file_path):
        sprite_types = []

        with open(file_path, 'r') as file:
            file_contents = file.read()

        # Extract sprite type sections
        sprite_type_sections = re.findall(r'SpriteType = \{(.+?)\}', file_contents, re.DOTALL)

        for section in sprite_type_sections:
            # Extract name
            name_match = re.search(r'name\s*=\s*"(.+?)"', section)
            if name_match:
                sprite_name = name_match.group(1)
            else:
                continue  # Skip this section if name is not found

            # Extract texture file
            texture_file_match = re.search(r'texturefile\s*=\s*"(.+?)"', section)
            if texture_file_match:
                texture_file = texture_file_match.group(1)
                folder_path = os.path.splitext(texture_file)[0]
                file_name = texture_file.split('/')[-1].split('.')[0]
                sprite_types.append((sprite_name, folder_path, file_name))

        return sprite_types
    
    def find_remainder_files(discovered_files, sprite_types):
        # Create a list to store the remainder files
        remainder_files = []

        # Loop through the discovered files
        for file_name, file_path in discovered_files:
            matched = False
            for sprite_name, sprite_file_name, sprite_file_path in sprite_types:
                if file_name == sprite_file_name and file_path == sprite_file_path:
                    matched = True
                    break

            if not matched:
                # Append the file to the remainder list if no match was found
                remainder_files.append((file_name, file_path))

        return remainder_files

    #def generate_entries(self, file_list, output_file, template, mode):
    #    with open(output_file, 'w') as file:
    #        file.write("spriteTypes = {\n")
    #        if mode == True:
    #            for file_name, folder_path in file_list:
    #                sprite_name = self.generate_sprite_name(file_name, folder_path) #Pain. See Method below
    #                entry = template.replace('<sprite name>', sprite_name)
    #                entry = entry.replace('<folder>', folder_path)
    #                entry = entry.replace('<filename>', file_name)
    #                entry = entry.replace('//', '/')
    #                file.write(entry)
    #        else:
    #            for sprite_name, file_name, folder_path in file_list:
    #                entry = template.replace('<sprite name>', sprite_name)
    #                entry = entry.replace('<folder>', folder_path)
    #                entry = entry.replace('<filename>', file_name)
    #                entry = entry.replace('//', '/')
    #                file.write(entry)
    #        file.write("}")

    def generate_entries(self, file_list, output_file, template, mode=None, sprite_file=None):
        with open(output_file, 'w') as file:
            file.write("spriteTypes = {\n")
            if mode:
                if sprite_file:
                    sprite_types = self.parse_gfx_file(sprite_file)
                else:
                    raise ValueError("When mode is True, sprite_file must be provided.")
                
                for sprite_name, file_name, folder_path in file_list:
                    entry = template.replace('<sprite name>', sprite_name)
                    entry = entry.replace('<folder>', folder_path)
                    entry = entry.replace('<filename>', file_name)
                    entry = entry.replace('//', '/')
                    file.write(entry)

                # Find the remainder files and append to the output file
                remainder_files = self.find_remainder_files(file_list, sprite_types)
                for file_name_y, folder_path_y in remainder_files:
                    sprite_name_r = self.generate_sprite_name(file_name_y, folder_path_y)
                    entry = template.replace('<sprite name>', sprite_name_r)
                    entry = entry.replace('<folder>', folder_path_y)
                    entry = entry.replace('<filename>', file_name_y)  # Use file_name here
                    entry = entry.replace('//', '/')
                    file.write(entry)
            else:
                for file_name_z, folder_path_z in file_list:
                    sprite_name_s = self.generate_sprite_name(file_name_z, folder_path_z)
                    entry = template.replace('<sprite name>', sprite_name_s)
                    entry = entry.replace('<folder>', folder_path_z)
                    entry = entry.replace('<filename>', file_name_z)
                    entry = entry.replace('//', '/')
                    file.write(entry)

            file.write("}")

    def generate_sprite_name(self, file_name, folder_path): #Big O comes here to die
        sprite_name = os.path.splitext(file_name)[0]
        sprite_name_parts = [part.lower() for part in re.split(r'[-_ ]', sprite_name)]  #Slap Chop
        subfolder = folder_path.split('/')[-1]                              # Get the last part of the folder path
        if len(subfolder) == 3:
            subfolder = subfolder.upper()
        for i in range(min(2, len(sprite_name_parts))):
            if sprite_name_parts[i] == subfolder.lower():
                sprite_name_parts[i] = subfolder
                                                                            #OH BOY DO I LOVE IF STATEMENTS
        if "interface/ideas" in folder_path:                                #GFX_idea handling for ideas
            if sprite_name_parts[0].lower() in ["gfx", "idea", "ideas"]:
                sprite_name_parts[0] = "GFX_idea"
                if subfolder.isalnum() and len(subfolder) == 3:
                    if subfolder not in sprite_name_parts:
                        sprite_name_parts.insert(1, subfolder)
            elif subfolder.isalnum() and len(subfolder) == 3:
                if subfolder not in sprite_name_parts:
                    sprite_name_parts.insert(0, "GFX_idea")
                    sprite_name_parts.insert(1, subfolder)
                else:
                    sprite_name_parts.insert(0, "GFX_idea")  
            else:
                sprite_name_parts.insert(0, "GFX_idea")              
        elif sprite_name_parts[0].lower() in ["gfx"]:
            sprite_name_parts[0] = "GFX"
            if subfolder.isalnum() and len(subfolder) == 3:
                if subfolder not in sprite_name_parts:
                     sprite_name_parts.insert(0, subfolder)
        elif subfolder.isalnum() and len(subfolder) == 3:
            if subfolder not in sprite_name_parts:
                sprite_name_parts.insert(0, "GFX")
                sprite_name_parts.insert(1, subfolder)                
            else:
                sprite_name_parts.insert(0, "GFX")  
        else:
            sprite_name_parts.insert(0, "GFX")  
            subfolder = ""  # Make subfolder empty if it doesn't meet the criteria

        final_sprite_name = '_'.join(sprite_name_parts)  # Join the parts with underscores
        final_sprite_name = final_sprite_name.rstrip(string.punctuation)
        
        return final_sprite_name



    def generate_gfx_entries(self, args):
        if args.subfolder:
            subfolder_files = self.GFXfinder.find_files(subfolder=args.subfolder, extensions=['.dds', '.png'])
            output_filename = '{}.gfx'.format(os.path.basename(args.subfolder))
            self.generate_entries(subfolder_files, output_filename, self.spritetype_template, mode=args.remainder_mode, sprite_file=args.sprite_file_path)
            print('Successfully generated spritetype entries for {} files in the {} subfolder.'.format(len(subfolder_files), args.subfolder))
            if args.subfolder == 'interface/goals':
                self.generate_entries(subfolder_files, 'goals_shines.gfx', self.shines_template, mode=args.remainder_mode, sprite_file=args.sprite_file_path)
                print('Successfully generated shines entries for {} files in the {} subfolder.'.format(len(subfolder_files), args.subfolder))

        if args.goals_shines:
            goals_files = self.GFXfinder.find_files(subfolder='interface/goals', extensions='.dds')
            self.generate_entries(goals_files, 'goals.gfx', self.spritetype_template, mode=args.remainder_mode, sprite_file=args.sprite_file_path)
            self.generate_entries(goals_files, 'goals_shines.gfx', self.shines_template, mode=args.remainder_mode, sprite_file=args.sprite_file_path)
            print('Successfully generated goals and shines entries for {} files.'.format(len(goals_files)))

        if args.ideas:
            ideas_files = self.GFXfinder.find_files(subfolder='interface/ideas', extensions='.dds')
            self.generate_entries(ideas_files, 'ideas.gfx', self.spritetype_template, mode=args.remainder_mode, sprite_file=args.sprite_file_path)
            print('Successfully generated ideas entries for {} files.'.format(len(ideas_files)))

        if args.event_pictures:
            event_pictures_files = self.GFXfinder.find_files(subfolder='event_pictures', extensions='.dds')
            self.generate_entries(event_pictures_files, 'event_pictures.gfx', self.spritetype_template, mode=args.remainder_mode, sprite_file=args.sprite_file_path)
            print('Successfully generated event pictures entries for {} files.'.format(len(event_pictures_files)))

        if args.leader_gfx:
            leader_portrait_files = self.GFXfinder.find_files(subfolder='leaders', extensions='.dds')
            self.generate_entries(leader_portrait_files, 'leaders.gfx', self.spritetype_template, mode=args.remainder_mode, sprite_file=args.sprite_file_path)
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
    
    
    parser.add_argument('--remainder-mode', action='store_true',
                        help='enable remainder-mode for sprite generation keeping preexising sprite entries')
    parser.add_argument('--sprite-file-path', type=str, default=None,
                        help='path to the sprite file for mode-based sprite generation')
    
    args = parser.parse_args()

    generator = GFXEntryGenerator()
    generator.generate_gfx_entries(args)

if __name__ == '__main__':
    main()