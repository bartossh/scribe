import shlex
import subprocess

for _ in range(1000):
    cmd = '''
        curl http://localhost:8080/save -H "Content-Type: application/json" --request POST --data '{"log": "biskupa dupa w logach sobie siedzi bo czemu by nie, tak, a moze aaaa bbb"}' '''
    args = shlex.split(cmd)
    process = subprocess.Popen(args, shell=False, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, stderr = process.communicate()
    cmd = '''
       curl http://localhost:8080/read -H "Content-Type: application/json" --request POST --data '{"from": 0, "to": 1915064118302449000}'  '''
    args = shlex.split(cmd)
    process = subprocess.Popen(args, shell=False, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, stderr = process.communicate()