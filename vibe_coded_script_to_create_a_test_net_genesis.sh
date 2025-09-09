
# 1) Params
export GEN_TIME=1755602142
export GEN_BITS_HEX=207fffff
export GEN_PUBKEY=04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f
export GEN_MSG="SLnet genesis 0254"

# 2) Coinbase v1 with ONLY BIP34 height=0 (OP_0), no extra message
python3 - <<'PY' > /tmp/coinbase.hex
import struct, os

# Zebra's exact genesis coinbase data (77 bytes) — NO height prefix:
GENESIS_COINBASE_DATA = bytes([
    4,255,255,7,31,1,4,69,90,99,97,115,104,48,98,57,99,52,101,101,102,56,98,
    55,99,99,52,49,55,101,101,53,48,48,49,101,51,53,48,48,57,56,52,98,54,102,
    101,97,51,53,54,56,51,97,55,99,97,99,49,52,49,97,48,52,51,99,52,50,48,
    54,52,56,51,53,100,51,52,
])

def compact(n):
    if n < 0xfd:   return bytes([n])
    if n <= 0xffff: return b'\xfd'+struct.pack('<H', n)
    if n <= 0xffffffff: return b'\xfe'+struct.pack('<I', n)
    return b'\xff'+struct.pack('<Q', n)

# Build scriptSig as varint(77) + GENESIS_COINBASE_DATA
scriptsig = compact(len(GENESIS_COINBASE_DATA)) + GENESIS_COINBASE_DATA  # 0x4d + 77 bytes

# P2PK scriptPubKey to your pubkey (from env)
PUBKEY = os.environ.get("GEN_PUBKEY","").strip()
if not PUBKEY: raise SystemExit("GEN_PUBKEY env var not set")
pk  = bytes.fromhex(PUBKEY)
spk = bytes([len(pk)]) + pk + b'\xac'   # <65B pubkey> OP_CHECKSIG

# v1 coinbase: 1 vin (coinbase), 1 vout (0 value), locktime 0
tx = (
    struct.pack('<I', 1) +                  # nVersion
    b'\x01' +                               # vin count
    b'\x00'*32 + b'\xff\xff\xff\xff' +      # coinbase prevout
    scriptsig +                             # scriptSig (exact genesis data, NO height)
    b'\xff\xff\xff\xff' +                   # nSequence
    b'\x01' +                               # vout count
    struct.pack('<q', 0) +                  # value = 0
    compact(len(spk)) + spk +               # scriptPubKey
    struct.pack('<I', 0)                    # nLockTime
)
print(tx.hex())
PY


# 3) 140-byte header preimage (NU5-style layout: includes 32B commitments=0 + 32B nonce=0)
python3 - <<'PY' > /tmp/genesis_hdr140.hex
import struct, hashlib
def dsha(b): return hashlib.sha256(hashlib.sha256(b).digest()).digest()
import os
GEN_TIME=int(os.environ['GEN_TIME']); GEN_BITS=int(os.environ['GEN_BITS_HEX'],16)
VERSION=4; PREV=b'\x00'*32; BCOMMIT=b'\x00'*32; NONCE=b'\x00'*32
cb = bytes.fromhex(open("/tmp/coinbase.hex").read().strip())
merkle = dsha(cb)
hdr140 = (struct.pack("<I",VERSION)+PREV+merkle+BCOMMIT+struct.pack("<I",GEN_TIME)+struct.pack("<I",GEN_BITS)+NONCE)
print(hdr140.hex())
PY

# 4) Mine one solution fast, then stop (adjust -t for your CPU)
HDR140=$(cat /tmp/genesis_hdr140.hex)
../equihash/equi -x "$HDR140" -r 64 -t 7 -s | awk '/^Solution /{print; exit}' > /tmp/sol_indices.txt

# 5) Pack 512 indices -> 1344 bytes
python3 - <<'PY' > /tmp/sol1344.hex
import re
with open("/tmp/sol_indices.txt") as f:
    data = f.read()
vals=[int(x,16) for x in re.findall(r'[0-9a-fA-F]+',data)]
assert len(vals)==512 and all(v<(1<<21) for v in vals)
buf=0; bl=0; out=bytearray()
for v in vals:
    buf=(buf<<21)|v; bl+=21
    while bl>=8:
        out.append((buf>>(bl-8))&0xff)
        bl-=8
if bl:
    out.append((buf<<(8-bl))&0xff)
assert len(out)==1344
print(out.hex())
PY

# 6) Assemble full block (varint fd4005 + 1344B solution + 1 tx + coinbase) and print block hash
python3 - <<'PY'
import struct, hashlib
def varint(n): 
  return (bytes([n]) if n<0xfd else b'\xfd'+(n).to_bytes(2,'little'))
def dsha(b): return hashlib.sha256(hashlib.sha256(b).digest()).digest()
hdr140=bytes.fromhex(open("/tmp/genesis_hdr140.hex").read().strip())
sol=bytes.fromhex(open("/tmp/sol1344.hex").read().strip())
cb=bytes.fromhex(open("/tmp/coinbase.hex").read().strip())
block = hdr140 + varint(len(sol)) + sol + b'\x01' + cb
open("/tmp/genesis_block.hex","w").write(block.hex())
# Config 'genesis_hash' = double-SHA256 of the 80-byte header (version..nBits)
hdr80 = hdr140[:80]
print("GENESIS_HASH =", dsha(hdr80)[::-1].hex())
PY

# hash is wrong, this gives the real hash

python3 - <<'PY'
import hashlib, struct
def varint(n):
    if n<0xfd: return bytes([n])
    if n<=0xffff: return b'\xfd'+(n).to_bytes(2,'little')
    if n<=0xffffffff: return b'\xfe'+(n).to_bytes(4,'little')
    return b'\xff'+(n).to_bytes(8,'little')
hdr140 = bytes.fromhex(open("/tmp/genesis_hdr140.hex").read().strip())
sol    = bytes.fromhex(open("/tmp/sol1344.hex").read().strip())
header = hdr140 + varint(len(sol)) + sol
block_hash = hashlib.sha256(hashlib.sha256(header).digest()).digest()[::-1].hex()
print("Zebra_block_hash =", block_hash)
PY