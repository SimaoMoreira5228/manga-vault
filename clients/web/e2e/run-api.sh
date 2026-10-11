set -euo pipefail

e2e_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "${e2e_dir}/../../.." && pwd)"
data_dir="/tmp/mv-e2e"
seed_addr="127.0.0.1:8082"
api_addr="127.0.0.1:8081"

export PLUGINS_DIR="${e2e_dir}/fixtures/plugins"
export DATA_DIR="${data_dir}"
export DATABASE_URL="sqlite://${data_dir}/v.db?mode=rwc"
export BIND_ADDR="${api_addr}"
export ADMIN_USERNAME="admin"
export SECRET_KEY="32e3d482c7f895260a35307fa7b78236fe5234f8fbe596f060e72c82c7f36dc4"

rm -rf "${data_dir}"
mkdir -p "${data_dir}"

cargo build --manifest-path "${root}/Cargo.toml" -p manga-vault-server

seed_log="${data_dir}/seed-server.log"
BIND_ADDR="${seed_addr}" "${root}/target/debug/manga-vault" >"${seed_log}" 2>&1 &
seed_pid=$!

stop_seed() {
	kill "${seed_pid}" 2>/dev/null || true
	wait "${seed_pid}" 2>/dev/null || true
}
trap stop_seed EXIT

ready=0
for _ in $(seq 1 240); do
	if ! kill -0 "${seed_pid}" 2>/dev/null; then
		break
	fi
	if curl -s -o /dev/null "http://${seed_addr}/api/registration"; then
		ready=1
		break
	fi
	sleep 0.5
done
if [ "${ready}" -ne 1 ]; then
	cat "${seed_log}"
	exit 1
fi

E2E_API_URL="http://${seed_addr}" E2E_DB_PATH="${data_dir}/v.db" python3 "${e2e_dir}/fixtures/seed_works.py"

stop_seed
trap - EXIT

exec "${root}/target/debug/manga-vault"
