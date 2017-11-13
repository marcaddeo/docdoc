# docdoc
`docdoc` is a simple Markdown-to-themed-HTML document generator. It supports
[CommonMark][] Markdown, as well as [GitHub Flavored][] Markdown.

## Usage
```
docdoc

Generate a themed HTML document from Markdown. Supports both CommonMark and GitHub Flavored Markdown.

Usage:
    docdoc [options] [--] <file> <output-dir>
    docdoc -h | --help
    docdoc --version

Options:
    --theme=<theme>                 Use a custom theme. [default: /usr/local/share/docodoc/themes/default]
    --template=<template>           Use a specific template in a theme. [default: index.html]
    -p, --preserve-first-component  Don't strip out the first component of the document path.
    --gfm                           Use GitHub Flavored Markdown.
    -h, --help                      Show this screen.
    --version                       Show version.
```

## Themes
Themes follow a simple set of conventions using YAML for configuration.

### Example theme
```
example_theme
├── assets
│   └── css
│       └── styles.css
├── index.html
└── theme.yml
```

### The theme configuration file
The `theme.yml` file contains all configuration for the theme. The following
fields are required:

* `name` - This is the theme's name
* `metadata` - This defines all possible metadata that a template can use.
  Markdown documents may override the values defined in the theme using YAML
  frontmatter.
* `assets` - This is a list of asset files that should be copied to the output
  directory of the HTML document. This can be files or directories, but we're
  just using a single `assets` directory to contain all of our assets for
  simplicity.

**`example_theme/theme.yml`**
```yaml
name: example_theme

metadata:
    title: Example Document

assets:
    - assets
```

### Templates
`docdoc` uses the [Tera][] template engine. [Tera][] templates are much like
Jinja2 or Twig. Templates are named like `template_name.html`.

By default, `docdoc` will use `index.html` as the template when generating a
document.

**`example_theme/index.html`**
```jinja
<!doctype html>

<html lang="en">
<head>
    <meta charset="utf-8">

    <title>{{ document.metadata.title }}</title>
    <link rel="stylesheet" href="assets/css/styles.css">

    <!--[if lt IE 9]>
        <script src="https://cdnjs.cloudflare.com/ajax/libs/html5shiv/3.7.3/html5shiv.js"></script>
    <![endif]-->
</head>

<body>
    {{ document.body | safe }}
</body>
</html>
```

You may pass the `--template=template_name.html` to use an
alternative template for the document.

### The `document` object
Templates have access to a `document` context object. The following fields are
accessible from templates:

* `metadata` - This is hash of theme metadata. All possible metadata used
  within a template must be defined in the `theme.yml` file under the
  `metadata` field. A markdown document may override any metadata using YAML
  frontmatter.
* `body` - This is the HTML body representation of the Markdown document. It
  contains HTML, so you'll have to use the `| safe` filter to render it.

## Documents
Documents are written in Markdown and must use `.md` as an extension, and may
contain YAML frontmatter to specify document specific Markdown. For instance,
you may want to update the rendered documents `<title>` on a per document
basis.

Frontmatter must begin with a line containing only `---` and end with a line
containing only `---`.

**docs/example.md**
```markdown
---
title: Example Document Title
---
# Example document

This is an example document. It overrides the theme's `title` metadata with
it's own document specific version.
```

### Output
Documents will be written to the specified `<output-dir>`. Document paths will
be preserved, but the first component of the directory will be stripped off.

For simplicity, assets are copied alongside each document that gets generated.

For example:
`docdoc docs/example.md docs/dist`

Generates:
```
docs/dist
├── assets
│   └── css
│       └── styles.css
└── example.html
```

And `docdoc docs/some/path/example.md docs/dist` generates:
```
docs/dist/some/path
├── assets
│   └── css
│       └── styles.css
└── example.html
```

Leaving us with the final directory structure:
```
docs/dist
├── assets
│   └── css
│       └── styles.css
├── example.html
└── some
    └── path
        ├── assets
        │   └── css
        │       └── styles.css
        └── example.html
```

If you don't wish to strip off the first directory, you may pass the
`--preserve-first-component` or `-p` flags.

`docdoc --preserve-first-component docs/some/path/example.md docs/dist` will
generate:
```
docs/dist/docs/some/path
├── assets
│   └── css
│       └── styles.css
└── example.html
```

Note that `docs/` was not stripped from the output path.

[CommonMark]: http://commonmark.org/
[GitHub Flavored]: https://github.github.com/gfm/
[Tera]: https://tera.netlify.com/
