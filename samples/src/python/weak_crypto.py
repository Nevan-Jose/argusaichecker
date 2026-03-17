import hashlib

def hash_password(password: str) -> str:
    # MD5 is cryptographically broken - do not use for passwords
    return hashlib.md5(password.encode()).hexdigest()

def verify_password(password: str, stored_hash: str) -> bool:
    return hash_password(password) == stored_hash
