---
name: tauri.conf.json macOS fields
description: Correct field names for macOS bundle config in tauri.conf.json v2
type: reference
---

Under `bundle.macOS` in `tauri.conf.json`:

- Info.plist path: `"infoPlist"` (NOT `"infoPlistPath"`)
- Entitlements: `"entitlements"`

Example:
```json
"bundle": {
  "macOS": {
    "entitlements": "entitlements.plist",
    "infoPlist": "Info.plist"
  }
}
```

The field `infoPlistPath` does not exist in tauri-build v2 and will cause a build failure with an "unknown field" error listing the valid alternatives.
