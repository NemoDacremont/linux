# Projet 05 - Création d'un driver Linux pour carte réseau RTL8139 en C ou Rust

Le but de ce projet est de créer un driver pour la carte Ethernet RTL8139 en
Rust pour le *kernel* Linux. Pour ce faire, un driver en C pourra être écrit
pour tester et comprendre comment est structuré et s'écrit un tel driver.

## Compilation et lancement

Pour tester le driver, une machine virtuelle est créée avec `qemu`.

### Driver en `C`

Exécuter (depuis la racine du dépot) :

```sh
make -C pfa cstart
```

Ou depuis le dossier `/pfa` :

```sh
make cstart
```

### Driver en `Rust` (WIP)

Exécuter (depuis la racine du dépot) :

```sh
make -C pfa rstart
```

Ou depuis le dossier `/pfa` :

```sh
make rstart
```

## Étapes de la compilation

1. Compilation statique de `busybox` pour une architecture x86
2. Création d'un système de fichier en RAM `initramfs`, ayant un script `/init`
faisant appel aux exécutables de `busybox`
3. Compilation du kernel Linux en utilisant les bons drivers à l'aide de
configurations fixées

## Tests

Différents tests peuvent être effectués : tests de validations et tests de
formatage

### Tests de formatage

Pour le C (depuis le dossier `/pfa`) :

```sh
make c_testformat
```

Pour le Rust (depuis le dossier `/pfa`) :

```sh
make rust_testformat
```

### Tests de validation

Pour la v0 pour le C (depuis le dossier `/pfa`) :

```sh
make tval_v0
```
