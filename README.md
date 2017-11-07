# docdoc


## TODO
- [ ] Write README
- [ ] Create default theme
- [ ] Create examples (single doc, simple blog)
- [ ] Allow passing arbitrary data to themes via `-e` and `-e @file/path` to
  load in data from YAML/JSON source. `-e "document.title=Override Title"`,
  `-e "{"document": {"title": "Override Title"}}"`. This will need some
  thought, but should have similar functionality that Ansible has. `-e` vars
  should take precedence over everything else. `-e` > document > theme. NOTE:
  this will allow navigation to just be outlined in a yaml file or json. Also:
  Probably should support TOML and loading all files in a directory, e.g. `-e
  @../vars`. Maybe.
