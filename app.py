from flask import Flask, jsonify, request, render_template
import os

app = Flask(__name__)

def get_size(path):
    if os.path.isfile(path):
        return os.path.getsize(path)
    return sum(
        os.path.getsize(os.path.join(dp, f))
        for dp, _, files in os.walk(path) for f in files
    )

def scan_directory(path):
    return [
        {
            "Path": entry.path,
            "Name": os.path.basename(entry.path),
            "Size (Bytes)": get_size(entry.path),
            "Type": "Folder" if entry.is_dir() else "File"
        }
        for entry in os.scandir(path)
    ]

@app.route('/list', methods=['GET'])
def list_directory():
    path = request.args.get('path', '../OpenHome/OpenHome/dist/')
    if not os.path.exists(path):
        return jsonify({"error": "Path does not exist"}), 400
    
    data = scan_directory(path)
    return jsonify(data)

@app.route('/')
def index():
    return render_template('index.html')

if __name__ == '__main__':
    app.run(debug=True)
