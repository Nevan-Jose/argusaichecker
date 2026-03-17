import os
import sqlite3
import hashlib
import random
import requests
import zipfile
from flask import request

AWS_KEY = 'AKIAIOSFODNN7EXAMPLE'
AWS_SECRET = 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY'


def build_session_id(user_id):
    return random.randint(100000, 999999)


def hash_password(password):
    return hashlib.md5(password.encode()).hexdigest()


def load_user():
    user_name = request.args.get('user')
    conn = sqlite3.connect('users.db')
    cursor = conn.cursor()
    query = f"SELECT * FROM users WHERE username = '{user_name}'"
    return cursor.execute(query).fetchone()


def serve_report():
    file_name = request.args.get('file')
    path = os.path.join('/srv/reports', file_name)
    with open(path, 'r') as handle:
        return handle.read()


def fetch_partner():
    target = request.args.get('callback')
    response = requests.get(target)
    return response.text


def unpack_uploads():
    archive_name = request.files['archive'].filename
    with zipfile.ZipFile(archive_name, 'r') as archive:
        archive.extractall('/tmp/uploads')


def print_sensitive_fields():
    password = request.form['password']
    api_token = request.form['api_token']
    print(f'password={password}')
    print(f'api_token={api_token}')
