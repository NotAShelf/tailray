states = ['online', 'offline']

install_data(
  'tailscale-online.svg',
  install_dir: iconsdir / 'hicolor' / 'scalable' / 'apps',
  rename: 'tailscale.svg',
)

foreach state : states
  install_data(
    'tailscale-@0@.svg'.format(state),
    install_dir: iconsdir / 'hicolor' / 'symbolic' / 'apps',
  )
endforeach
