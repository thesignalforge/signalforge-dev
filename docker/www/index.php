<?php
/**
 * Signalforge Dev - Test Page
 */

$services = [
    'MySQL' => [
        'host' => 'mysql',
        'port' => 3306,
        'check' => function() {
            try {
                $pdo = new PDO(
                    'mysql:host=mysql;port=3306;dbname=' . ($_ENV['MYSQL_DATABASE'] ?? 'app'),
                    $_ENV['MYSQL_USER'] ?? 'dev',
                    $_ENV['MYSQL_PASSWORD'] ?? 'dev',
                    [PDO::ATTR_TIMEOUT => 2]
                );
                return ['status' => 'connected', 'version' => $pdo->query('SELECT VERSION()')->fetchColumn()];
            } catch (Exception $e) {
                return ['status' => 'error', 'message' => $e->getMessage()];
            }
        }
    ],
    'PostgreSQL' => [
        'host' => 'postgres',
        'port' => 5432,
        'check' => function() {
            try {
                $pdo = new PDO(
                    'pgsql:host=postgres;port=5432;dbname=' . ($_ENV['POSTGRES_DB'] ?? 'app'),
                    $_ENV['POSTGRES_USER'] ?? 'dev',
                    $_ENV['POSTGRES_PASSWORD'] ?? 'dev',
                    [PDO::ATTR_TIMEOUT => 2]
                );
                return ['status' => 'connected', 'version' => $pdo->query('SELECT VERSION()')->fetchColumn()];
            } catch (Exception $e) {
                return ['status' => 'error', 'message' => $e->getMessage()];
            }
        }
    ],
    'Redis' => [
        'host' => 'redis',
        'port' => 6379,
        'check' => function() {
            try {
                $redis = new Redis();
                $redis->connect('redis', 6379, 2);
                return ['status' => 'connected', 'version' => $redis->info()['redis_version'] ?? 'unknown'];
            } catch (Exception $e) {
                return ['status' => 'error', 'message' => $e->getMessage()];
            }
        }
    ]
];

$results = [];
foreach ($services as $name => $service) {
    $results[$name] = $service['check']();
}
?>
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Signalforge Dev</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Monaco', 'Consolas', monospace;
            background: #050810;
            color: #e0e7f1;
            min-height: 100vh;
            padding: 40px;
        }
        .container { max-width: 1000px; margin: 0 auto; }
        h1 {
            color: #00d9ff;
            font-size: 2rem;
            margin-bottom: 8px;
            text-shadow: 0 0 20px rgba(0, 217, 255, 0.5);
        }
        .subtitle { color: #7a8a9e; margin-bottom: 40px; }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; }
        .card {
            background: #121820;
            border: 1px solid #1f2937;
            border-radius: 8px;
            padding: 20px;
        }
        .card-title {
            font-size: 0.75rem;
            color: #00d9ff;
            letter-spacing: 2px;
            margin-bottom: 12px;
        }
        .status {
            display: flex;
            align-items: center;
            gap: 8px;
            margin-bottom: 8px;
        }
        .status-dot {
            width: 10px;
            height: 10px;
            border-radius: 50%;
        }
        .status-dot.connected { background: #00ff88; box-shadow: 0 0 10px rgba(0, 255, 136, 0.5); }
        .status-dot.error { background: #ff3344; box-shadow: 0 0 10px rgba(255, 51, 68, 0.5); }
        .version { color: #7a8a9e; font-size: 0.85rem; }
        .error-msg { color: #ff3344; font-size: 0.75rem; margin-top: 8px; }
        .php-info {
            background: #121820;
            border: 1px solid #1f2937;
            border-radius: 8px;
            padding: 20px;
            margin-top: 20px;
        }
        .info-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
        .info-item { padding: 8px 0; border-bottom: 1px solid #1f2937; }
        .info-label { color: #7a8a9e; font-size: 0.75rem; }
        .info-value { color: #e0e7f1; margin-top: 4px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Signalforge Dev</h1>
        <p class="subtitle">Development Environment Status</p>

        <div class="grid">
            <?php foreach ($results as $name => $result): ?>
            <div class="card">
                <div class="card-title"><?= strtoupper($name) ?></div>
                <div class="status">
                    <div class="status-dot <?= $result['status'] ?>"></div>
                    <span><?= ucfirst($result['status']) ?></span>
                </div>
                <?php if ($result['status'] === 'connected'): ?>
                    <div class="version"><?= htmlspecialchars($result['version'] ?? '') ?></div>
                <?php else: ?>
                    <div class="error-msg"><?= htmlspecialchars($result['message'] ?? 'Unknown error') ?></div>
                <?php endif; ?>
            </div>
            <?php endforeach; ?>
        </div>

        <div class="php-info">
            <div class="card-title">PHP ENVIRONMENT</div>
            <div class="info-grid">
                <div class="info-item">
                    <div class="info-label">PHP VERSION</div>
                    <div class="info-value"><?= PHP_VERSION ?></div>
                </div>
                <div class="info-item">
                    <div class="info-label">MEMORY LIMIT</div>
                    <div class="info-value"><?= ini_get('memory_limit') ?></div>
                </div>
                <div class="info-item">
                    <div class="info-label">MAX UPLOAD</div>
                    <div class="info-value"><?= ini_get('upload_max_filesize') ?></div>
                </div>
                <div class="info-item">
                    <div class="info-label">LOADED EXTENSIONS</div>
                    <div class="info-value"><?= count(get_loaded_extensions()) ?> extensions</div>
                </div>
            </div>
        </div>
    </div>
</body>
</html>
