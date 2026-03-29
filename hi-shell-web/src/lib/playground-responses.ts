export interface PlaygroundResponse {
	keywords: string[];
	command: string;
	dangerous: boolean;
	output: string;
}

const responses: PlaygroundResponse[] = [
	{
		keywords: ['list', 'files', 'ls'],
		command: 'ls -lah',
		dangerous: false,
		output: 'total 42\ndrwxr-xr-x  5 user  staff   160B Mar 28 10:00 .\ndrwxr-xr-x  3 user  staff    96B Mar 28 09:00 ..\n-rw-r--r--  1 user  staff  1.2K Mar 28 10:00 README.md\ndrwxr-xr-x  8 user  staff   256B Mar 28 10:00 src\n-rw-r--r--  1 user  staff   512B Mar 28 10:00 Cargo.toml'
	},
	{
		keywords: ['find', 'png', 'image', 'images'],
		command: 'find . -name "*.png" -type f',
		dangerous: false,
		output: './assets/logo.png\n./images/hero.png\n./public/favicon.png\n./static/og-image.png'
	},
	{
		keywords: ['find', 'large', 'big', 'size', 'mb'],
		command: 'find . -type f -size +10M -exec ls -lh {} \\;',
		dangerous: false,
		output: '-rw-r--r--  1 user  staff   45M Mar 20 14:22 ./data/dataset.json\n-rw-r--r--  1 user  staff   12M Mar 18 09:15 ./assets/video.mp4'
	},
	{
		keywords: ['docker', 'containers', 'running'],
		command: 'docker ps',
		dangerous: false,
		output: 'CONTAINER ID   IMAGE          STATUS          PORTS                    NAMES\na1b2c3d4e5f6   nginx:latest   Up 2 hours      0.0.0.0:80->80/tcp       web-server\nf6e5d4c3b2a1   redis:7        Up 5 hours      0.0.0.0:6379->6379/tcp   cache'
	},
	{
		keywords: ['docker', 'all', 'containers'],
		command: 'docker ps -a',
		dangerous: false,
		output: 'CONTAINER ID   IMAGE          STATUS                     NAMES\na1b2c3d4e5f6   nginx:latest   Up 2 hours                 web-server\nf6e5d4c3b2a1   redis:7        Up 5 hours                 cache\n7a8b9c0d1e2f   postgres:15    Exited (0) 3 hours ago     database'
	},
	{
		keywords: ['docker', 'stop', 'all'],
		command: 'docker stop $(docker ps -q)',
		dangerous: true,
		output: 'a1b2c3d4e5f6\nf6e5d4c3b2a1'
	},
	{
		keywords: ['git', 'status'],
		command: 'git status',
		dangerous: false,
		output: 'On branch main\nYour branch is up to date with \'origin/main\'.\n\nChanges not staged for commit:\n  modified:   src/main.rs\n  modified:   Cargo.toml\n\nUntracked files:\n  src/new_module.rs'
	},
	{
		keywords: ['git', 'log', 'history', 'recent', 'commits'],
		command: 'git log --oneline -10',
		dangerous: false,
		output: 'a1b2c3d feat: add interactive REPL mode\nb2c3d4e fix: handle UTF-8 in output\nc3d4e5f refactor: extract service layer\nd4e5f6a docs: update installation guide\ne5f6a7b feat: add cloud provider support\nf6a7b8c fix: config loading edge cases\na7b8c9d chore: update dependencies\nb8c9d0e feat: embedded model support\nc9d0e1f initial release\n'
	},
	{
		keywords: ['git', 'branch', 'branches'],
		command: 'git branch -a',
		dangerous: false,
		output: '* main\n  develop\n  feature/repl-mode\n  remotes/origin/main\n  remotes/origin/develop'
	},
	{
		keywords: ['disk', 'space', 'usage', 'df'],
		command: 'df -h',
		dangerous: false,
		output: 'Filesystem      Size   Used  Avail Capacity  Mounted on\n/dev/disk1     466Gi  234Gi  232Gi    50%    /\ntmpfs           1.0Gi  120Mi  904Mi    12%    /tmp'
	},
	{
		keywords: ['memory', 'ram', 'free'],
		command: 'free -h 2>/dev/null || vm_stat',
		dangerous: false,
		output: 'Mach Virtual Memory Statistics: (page size of 16384 bytes)\nPages free:                    142586\nPages active:                  398201\nPages inactive:                321554\nPages speculative:              18423'
	},
	{
		keywords: ['process', 'cpu', 'top', 'running'],
		command: 'ps aux --sort=-%cpu | head -15',
		dangerous: false,
		output: 'USER       PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND\nuser      1234 45.2  2.1 1234567 98765 ?        Sl   10:00  12:34 cargo build --release\nuser      5678 12.1  1.5  456789 67890 ?        Sl   10:15   3:21 node server.js\nuser      9012  3.4  0.8  234567 34567 ?        S    10:30   0:45 ollama serve'
	},
	{
		keywords: ['port', 'listening', 'network', 'open'],
		command: 'lsof -i -P -n | grep LISTEN',
		dangerous: false,
		output: 'nginx     1234 user    6u  IPv4 0x1234      TCP *:80 (LISTEN)\nnode      5678 user   12u  IPv4 0x5678      TCP *:3000 (LISTEN)\nredis     9012 user    4u  IPv4 0x9012      TCP 127.0.0.1:6379 (LISTEN)'
	},
	{
		keywords: ['download', 'curl', 'wget', 'fetch'],
		command: 'curl -L -o output.zip https://example.com/file.zip',
		dangerous: false,
		output: '  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current\n                                 Dload  Upload   Total   Spent    Left  Speed\n100  123M  100  123M    0     0  12.3M      0  0:00:10  0:00:10 12.3M'
	},
	{
		keywords: ['compress', 'zip', 'archive', 'tar'],
		command: 'tar -czf archive.tar.gz ./project',
		dangerous: false,
		output: ''
	},
	{
		keywords: ['extract', 'unzip', 'untar', 'decompress'],
		command: 'tar -xzf archive.tar.gz',
		dangerous: false,
		output: ''
	},
	{
		keywords: ['install', 'package', 'npm', 'cargo', 'brew'],
		command: 'npm install express',
		dangerous: false,
		output: 'added 57 packages in 3s\n\n14 packages are looking for funding\n  run `npm fund` for details'
	},
	{
		keywords: ['search', 'text', 'find', 'grep', 'string', 'file', 'containing'],
		command: 'grep -rn "search_term" ./src/',
		dangerous: false,
		output: './src/main.rs:42:    let search_term = args.query;\n./src/lib.rs:15:    pub fn search(query: &str) -> Vec<Result> {'
	},
	{
		keywords: ['kill', 'process', 'pid'],
		command: 'kill -9 1234',
		dangerous: true,
		output: ''
	},
	{
		keywords: ['delete', 'remove', 'rm', 'file', 'files'],
		command: 'rm -rf ./node_modules/',
		dangerous: true,
		output: ''
	},
	{
		keywords: ['delete', 'remove', 'log', 'logs'],
		command: 'find . -name "*.log" -type f -delete',
		dangerous: true,
		output: ''
	},
	{
		keywords: ['ssh', 'connect', 'server', 'remote'],
		command: 'ssh user@server.example.com',
		dangerous: false,
		output: 'Welcome to Ubuntu 22.04.3 LTS (GNU/Linux 5.15.0-91-generic x86_64)\n\nLast login: Thu Mar 28 09:00:00 2024 from 192.168.1.100'
	},
	{
		keywords: ['env', 'environment', 'variable', 'variables', 'set'],
		command: 'env | sort',
		dangerous: false,
		output: 'HOME=/Users/user\nLANG=en_US.UTF-8\nPATH=/usr/local/bin:/usr/bin:/bin\nSHELL=/bin/zsh\nTERM=xterm-256color\nUSER=user'
	},
	{
		keywords: ['who', 'am', 'i', 'user', 'logged'],
		command: 'whoami && id',
		dangerous: false,
		output: 'user\nuid=501(user) gid=20(staff) groups=20(staff),12(everyone),61(localaccounts)'
	}
];

export function findResponse(input: string): PlaygroundResponse | null {
	const lower = input.toLowerCase().trim();
	const words = lower.split(/\s+/);

	let bestMatch: PlaygroundResponse | null = null;
	let bestScore = 0;

	for (const response of responses) {
		let score = 0;
		for (const keyword of response.keywords) {
			if (words.includes(keyword) || lower.includes(keyword)) {
				score++;
			}
		}
		if (score > bestScore) {
			bestScore = score;
			bestMatch = response;
		}
	}

	if (bestScore >= 1) {
		return bestMatch;
	}

	return {
		keywords: [],
		command: `echo "Your request: ${input.replace(/"/g, '\\"')}"`,
		dangerous: false,
		output: `hi-shell understood: "${input}"\n\n(This is a simulated demo. Install hi-shell for real AI-powered command generation.)`
	};
}
