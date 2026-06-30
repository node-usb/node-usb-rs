import fs from 'node:fs'

const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'))
let cargo = fs.readFileSync('Cargo.toml', 'utf8')

cargo = cargo.replace(
  /^version\s*=\s*"[^"]+"/m,
  `version = "${pkg.version}"`
)

fs.writeFileSync('Cargo.toml', cargo)
