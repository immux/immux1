Steward
=======

Steward is the immutable computing service built upon ImmuxDB.

Development
-----------

### Initialization

1. set DNS to redirect foldr.foldr.test to localhost. (The easiest way would be adding `foldr.foldr.test 127.0.0.1` to `/etc/hosts`.)

2. Install Rust compiler (if you haven't already).

3. `npm install`

### Start

```bash
./dev.sh
```

### Reformat TypeScript

```bash
./formatts.sh
```

### Deploy

```bash
# Merge changes to master first
./deploy.sh
```

(Ask Andy for server access)

Architecture
------------

Steward is the basis that is responsible for holding JavaScript runtime. This system is marketed as "foldr".

`projects/foldr` is the UI and management logic of "Steward". It is a normal project running on Steward.
