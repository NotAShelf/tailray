project(
  'tailray',
  'rust',
  version: '0.2.1',
  license: 'MIT',
  meson_version: '>=0.59.0'
)

cargo = find_program('cargo', required: true)

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
datadir = prefix / get_option('datadir')
iconsdir = datadir / 'icons'

subdir('src')
subdir('icons')
