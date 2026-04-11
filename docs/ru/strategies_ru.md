# Стратегии маскирования

## Replace — полная замена

```
Input:  "Email: john@test.com, AWS Key: AKIAIOSFODNN7EXAMPLE"
Output: "Email: [EMAIL_1], AWS Key: [API_KEY_2]"
```

**Плюсы**: Полная анонимность, легко подсчитать PII
**Минусы**: Теряется контекст данных

## Mask — частичная маска

```
Email:    "john.doe@company.org"  →  "jo***@***rg"
Phone:    "+7 (999) 123-45-67"    →  "+79***67"
API Key:  "AKIAIOSFODNN7EXAMPLE"  →  "AKIA***MPLE"
Domain:   "secret-server.ru"      →  "sec***.ru"
```

**Плюсы**: Сохраняется формат данных
**Минусы**: Частичное раскрытие информации

## Hash — частичный хеш

```
Email:    "john.doe@company.org"  →  "jo_4f2a8b1c@om"
Phone:    "+7 (999) 123-45-67"    →  "+79_8e3f2a1d67"
API Key:  "AKIAIOSFODNN7EXAMPLE"  →  "AKIA_4f2a8bMPLE"
```

**Плюсы**: Необратимость, сохраняется структура
**Минусы**: Хеш может быть подобран перебором для коротких значений

## Примеры API ключей и токенов

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

### SSH ключи
```
Input:  "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDVvvHkGphJbBX8"
Mask:   "ssh-rsa AAAA***BX8"
```

### Домены (с фильтрацией)
```
Input:  "Search on google.com or visit secret-server.ru"
Output: "Search on google.com or visit secr***.ru"
# google.com пропущен — известный домен
```
