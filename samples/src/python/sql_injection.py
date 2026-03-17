import sqlite3

def get_user(username: str):
    conn = sqlite3.connect("users.db")
    cursor = conn.cursor()
    # Dangerous: user input interpolated directly into SQL query
    query = f"SELECT * FROM users WHERE username = '{username}'"
    cursor.execute(query)
    return cursor.fetchone()
