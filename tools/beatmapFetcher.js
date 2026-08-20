const fs = require('fs');
const path = require('path');
const https = require('https');
const http = require('http');

const CACHE_DIR = path.resolve(__dirname, '../cache');

if (!fs.existsSync(CACHE_DIR)) {
  fs.mkdirSync(CACHE_DIR, { recursive: true });
}

/**
 * Extracts a beatmap ID from a URL or raw ID string.
 * Supports:
 * - https://osu.ppy.sh/beatmapsets/2093204#osu/4388676
 * - https://osu.ppy.sh/b/4388676
 * - https://osu.ppy.sh/beatmaps/4388676
 * - 4388676
 * - Local file path (returns null for ID, but recognizes local file)
 */
function extractBeatmapId(input) {
  if (!input) return null;
  const str = input.toString().trim();

  // If it's an existing file path, return null ID (it's a direct file)
  if (fs.existsSync(str) && fs.statSync(str).isFile()) {
    return { isFile: true, filePath: path.resolve(str), id: null };
  }

  // Regex patterns
  // Pattern 1: #osu/123456 or #taiko/123456 or #fruits/123456 or #mania/123456
  const hashMatch = str.match(/#(?:osu|taiko|fruits|mania|catch)\/(\d+)/i);
  if (hashMatch) return { isFile: false, filePath: null, id: hashMatch[1] };

  // Pattern 2: /b/123456 or /beatmaps/123456
  const bMatch = str.match(/(?:\/b\/|\/beatmaps\/)(\d+)/i);
  if (bMatch) return { isFile: false, filePath: null, id: bMatch[1] };

  // Pattern 3: Pure numeric ID
  if (/^\d+$/.test(str)) {
    return { isFile: false, filePath: null, id: str };
  }

  return null;
}

/**
 * Download a file via HTTPS following redirects
 */
function downloadFile(url) {
  return new Promise((resolve, reject) => {
    const protocol = url.startsWith('https') ? https : http;
    const req = protocol.get(url, { headers: { 'User-Agent': 'sengo-pp-tester/1.0' } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return resolve(downloadFile(res.headers.location));
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`Failed to download beatmap from ${url} (Status: ${res.statusCode})`));
      }
      const chunks = [];
      res.on('data', chunk => chunks.push(chunk));
      res.on('end', () => resolve(Buffer.concat(chunks)));
    });
    req.on('error', reject);
  });
}

/**
 * Parses basic metadata from .osu content buffer
 */
function parseBeatmapMetadata(buffer) {
  const content = buffer.toString('utf8');
  const lines = content.split(/\r?\n/);
  const metadata = {
    title: 'Unknown Title',
    artist: 'Unknown Artist',
    version: 'Unknown Difficulty',
    creator: 'Unknown Mapper',
    mode: 0,
    cs: 5,
    ar: 5,
    od: 5,
    hp: 5,
    bpm: 120
  };

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith('Title:')) metadata.title = trimmed.slice(6).trim();
    else if (trimmed.startsWith('Artist:')) metadata.artist = trimmed.slice(7).trim();
    else if (trimmed.startsWith('Version:')) metadata.version = trimmed.slice(8).trim();
    else if (trimmed.startsWith('Creator:')) metadata.creator = trimmed.slice(8).trim();
    else if (trimmed.startsWith('Mode:')) metadata.mode = parseInt(trimmed.slice(5).trim(), 10) || 0;
    else if (trimmed.startsWith('CircleSize:')) metadata.cs = parseFloat(trimmed.slice(11).trim()) || 5;
    else if (trimmed.startsWith('ApproachRate:')) metadata.ar = parseFloat(trimmed.slice(13).trim()) || 5;
    else if (trimmed.startsWith('OverallDifficulty:')) metadata.od = parseFloat(trimmed.slice(18).trim()) || 5;
    else if (trimmed.startsWith('HPDrainRate:')) metadata.hp = parseFloat(trimmed.slice(12).trim()) || 5;
  }

  return metadata;
}

/**
 * Retrieves a beatmap by URL, ID, or file path.
 * Caches automatically in ./cache/<id>.osu
 */
async function getBeatmap(input) {
  const parsed = extractBeatmapId(input);
  if (!parsed) {
    throw new Error(`Could not parse beatmap ID or file from input: "${input}"`);
  }

  if (parsed.isFile) {
    const buffer = fs.readFileSync(parsed.filePath);
    const metadata = parseBeatmapMetadata(buffer);
    return {
      id: parsed.id,
      filePath: parsed.filePath,
      buffer,
      metadata
    };
  }

  const id = parsed.id;
  const cachedPath = path.join(CACHE_DIR, `${id}.osu`);

  if (fs.existsSync(cachedPath)) {
    const buffer = fs.readFileSync(cachedPath);
    const metadata = parseBeatmapMetadata(buffer);
    return {
      id,
      filePath: cachedPath,
      buffer,
      metadata,
      fromCache: true
    };
  }

  // Download directly from official osu! CDN
  const url = `https://osu.ppy.sh/osu/${id}`;
  const buffer = await downloadFile(url);

  // Validate that it looks like an osu file
  const header = buffer.slice(0, 30).toString('utf8');
  if (!header.includes('osu file format')) {
    throw new Error(`Downloaded content for ID ${id} is not a valid .osu file!`);
  }

  fs.writeFileSync(cachedPath, buffer);
  const metadata = parseBeatmapMetadata(buffer);

  return {
    id,
    filePath: cachedPath,
    buffer,
    metadata,
    fromCache: false
  };
}

module.exports = {
  extractBeatmapId,
  getBeatmap,
  parseBeatmapMetadata,
  CACHE_DIR
};
