import os

target = os.environ.get('SKILL_PARAM_TARGET_FILE', '')
print(f'Starting review on {target}')
print('Done.')
