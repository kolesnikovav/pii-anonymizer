# Masking Strategies

## Replace -- Full Replacement

```
Input:  "Email: john@test.com, AWS Key: AKIAIOSFODNN7EXAMPLE"
Output: "Email: [EMAIL_1], AWS Key: [API_KEY_2]"
```

**Pros**: Full anonymity, easy to count PII occurrences
**Cons**: Data context is lost

## Mask -- Partial Masking

```
Email:    "john.doe@company.org"  ->  "jo***@***rg"
Phone:    "+7 (999) 123-45-67"    ->  "+79***67"
API Key:  "AKIAIOSFODNN7EXAMPLE"  ->  "AKIA***MPLE"
Domain:   "secret-server.ru"      ->  "sec***.ru"
```

**Pros**: Data format is preserved
**Cons**: Partial information disclosure

## Hash -- Partial Hash

```
Email:    "john.doe@company.org"  ->  "jo_4f2a8b1c@om"
Phone:    "+7 (999) 123-45-67"    ->  "+79_8e3f2a1d67"
API Key:  "AKIAIOSFODNN7EXAMPLE"  ->  "AKIA_4f2a8bMPLE"
```

**Pros**: Irreversible, structure is preserved
**Cons**: Hash can be brute-forced for short values

## API Keys and Tokens Examples

### AWS
```
Input:  "My AWS key is AKIAIOSFODNN7EXAMPLE"
Mask:   "My AWS key is AKIA***MPLE"
Replace: "My AWS key is [API_KEY_1]"
```

### GitHub
```
Input:  "Token: ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefgh12"
Mask:   "Token: ghp_***h12"
```

### JWT
```
Input:  "Bearer eyJhbGciOiJIUzI1NiIs..."
Mask:   "Bearer eyJ_***..."
```

### SSH Keys
```
Input:  "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDVvvHkGphJbBX8"
Mask:   "ssh-rsa AAAA***BX8"
```

### Domains (with filtering)
```
Input:  "Search on google.com or visit secret-server.ru"
Output: "Search on google.com or visit secr***.ru"
# google.com is skipped -- known domain
```
