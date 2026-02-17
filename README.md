# shutters-service

Petit service/daemon Rust pour piloter les volets (squelette).

## Description

`shutters-service` est un binaire Rust minimal servant de point d'entrée pour
le code qui devra piloter les volets (GPIO, IPC, etc.). Le service :

- initialise le logger (`env_logger`),
- gère l'arrêt propre via `ctrl+c`,
- exécute une boucle périodique (emplacement pour le "vrai" travail).

## Prérequis

- Rust toolchain (stable) et `cargo`.
- Avoir une cle OPENWEATHER_API_KEY dans les variable d'env (soit via la conf du service soit via un export dans un sh)

## Build & exécution

Depuis le répertoire `shutters-service` :

```bash
cargo build --release
# exécution de développement
cargo run
```

Pour un déploiement sur la machine cible, utilisez l'exécutable dans :

```text
target\release\shutters-service
```

## Emplacement du code

- Point d'entrée : `src/main.rs` (boucle principale, gestion du signal d'arrêt).
- Manifest : `Cargo.toml`.

## Comportement actuel

La boucle principale se contente pour l'instant d'écrire un message de log
toutes les 5 secondes :

```rust
// TODO: replace with real work (IPC, GPIO, etc.)
```

Remplacez cette portion par la logique métier : communication avec le driver,
gestion des broches GPIO ou exposition d'une API IPC/HTTP.

## Logging

Le projet utilise `env_logger`. Contrôlez le niveau de log via la variable
d'environnement `RUST_LOG` :

```bash
RUST_LOG=info cargo run
RUST_LOG=debug ./target/release/shutters-service
```

## Arrêt propre

Le signal d'arrêt (`Ctrl+C`) est géré et provoque l'arrêt gracieux de la boucle.

## Prochaines étapes suggérées

- Implémenter l'interface de contrôle réelle (GPIO / driver / IPC).
- Ajouter des tests d'intégration et une configuration (fichier TOML/env).
- Fournir un unit/service file si vous voulez lancer en tant que service système.

## Lancer en tant que service système (systemd)

Un unit `systemd` est fourni dans `deploy/shutters-service.service`. Exemple d'installation :

```bash
# construire en release
cargo build --release

# créer un utilisateur système (optionnel mais recommandé)
sudo useradd -r -s /usr/sbin/nologin shutters

# installer l'exécutable et les fichiers
sudo mkdir -p /opt/shutters-service
sudo cp target/release/shutters-service /opt/shutters-service/
sudo chown -R shutters:shutters /opt/shutters-service

# copier l'unit et démarrer le service
sudo cp deploy/shutters-service.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now shutters-service

# voir les logs
sudo journalctl -u shutters-service -f
```

Ajustez les chemins dans `deploy/shutters-service.service` si vous installez ailleurs. Le service redémarre automatiquement en cas d'échec (`Restart=on-failure`).

Note: si vous ciblez une machine Windows, utilisez un gestionnaire de services natif (par ex. `nssm`) ou installez comme service via des wrappers spécifiques Windows.

## Script d'administration

Un script utilitaire est fourni : `deploy/manage_service.sh`. Il facilite l'installation et la gestion du service :

```bash
# installer (construit en release, copie le binaire, installe l'unit et démarre)
./deploy/manage_service.sh install

# contrôler le service
./deploy/manage_service.sh start
./deploy/manage_service.sh stop
./deploy/manage_service.sh restart
./deploy/manage_service.sh status

# désinstaller
./deploy/manage_service.sh uninstall
```

Le script utilise `sudo` pour les opérations système. Vérifiez et adaptez les chemins (`/opt/shutters-service`) si nécessaire.
