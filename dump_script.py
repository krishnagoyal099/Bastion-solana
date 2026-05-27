import os

out_path = '/home/starlord/bastion/codebase_dump.txt'
exclude_dirs = {'landing-page', '.git', 'node_modules', 'target', 'build', 'dist'}

with open(out_path, 'w', encoding='utf-8') as out_f:
    for root, dirs, files in os.walk('/home/starlord/bastion'):
        # Filter directories in-place
        dirs[:] = [d for d in dirs if d not in exclude_dirs and not d.startswith('.')]
        for file in files:
            if file.startswith('.'):
                continue
            if file in ['codebase_dump.txt', 'dump_script.py', 'package-lock.json', 'yarn.lock', 'Cargo.lock']:
                continue
                
            filepath = os.path.join(root, file)
            
            # Simple check to avoid very large files, e.g., > 1MB
            if os.path.getsize(filepath) > 1024 * 1024:
                continue
                
            out_f.write(f'{"="*80}\n')
            out_f.write(f'File: {os.path.relpath(filepath, "/home/starlord/bastion")}\n')
            out_f.write(f'{"="*80}\n')
            try:
                with open(filepath, 'r', encoding='utf-8') as in_f:
                    out_f.write(in_f.read())
            except UnicodeDecodeError:
                out_f.write('[Binary file or non-UTF8 content]\n')
            except Exception as e:
                out_f.write(f'[Error reading file: {e}]\n')
            out_f.write('\n\n')
