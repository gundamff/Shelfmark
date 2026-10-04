# Shelfmark rule packs

Drop JSON files into `%AppData%/Shelfmark/rule-packs/` to extend AutoCat without rebuilding Shelfmark.

See built-in examples in repo `rule-packs/`. Unknown `kind` values fail that pack at load/preview time.

Supported kinds: `genre`, `tags`, `year`, `flags`, `devpub`, `user_score`.
