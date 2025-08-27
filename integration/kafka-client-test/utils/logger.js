/**
 * Enhanced Logger for Learning
 * 
 * This logger is designed to make the learning experience better
 * by providing clear, color-coded, and structured output that
 * helps students understand what's happening at each step.
 */

// ANSI color codes for better visual learning
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m',
  bgRed: '\x1b[41m',
  bgGreen: '\x1b[42m',
  bgYellow: '\x1b[43m',
  bgBlue: '\x1b[44m'
};

/**
 * Format timestamp for logs
 */
function getTimestamp() {
  return new Date().toISOString().replace('T', ' ').split('.')[0];
}

/**
 * Enhanced logger with educational focus
 */
const logger = {
  /**
   * Info level logging (general information)
   */
  info: (message, ...args) => {
    console.log(`${colors.cyan}[INFO]${colors.reset} ${colors.dim}${getTimestamp()}${colors.reset} ${message}`, ...args);
  },

  /**
   * Success level logging (positive outcomes)
   */
  success: (message, ...args) => {
    console.log(`${colors.green}[SUCCESS]${colors.reset} ${colors.dim}${getTimestamp()}${colors.reset} ${message}`, ...args);
  },

  /**
   * Warning level logging (potential issues)
   */
  warning: (message, ...args) => {
    console.log(`${colors.yellow}[WARNING]${colors.reset} ${colors.dim}${getTimestamp()}${colors.reset} ${message}`, ...args);
  },

  /**
   * Error level logging (actual problems)
   */
  error: (message, ...args) => {
    console.error(`${colors.red}[ERROR]${colors.reset} ${colors.dim}${getTimestamp()}${colors.reset} ${message}`, ...args);
  },

  /**
   * Debug level logging (detailed technical info)
   */
  debug: (message, ...args) => {
    if (process.env.DEBUG || process.env.NODE_ENV === 'development') {
      console.log(`${colors.magenta}[DEBUG]${colors.reset} ${colors.dim}${getTimestamp()}${colors.reset} ${message}`, ...args);
    }
  },

  /**
   * Learning tip logging (educational insights)
   */
  tip: (message, ...args) => {
    console.log(`${colors.blue}💡 [TIP]${colors.reset} ${colors.dim}${getTimestamp()}${colors.reset} ${message}`, ...args);
  },

  /**
   * Step logging (for sequential processes)
   */
  step: (stepNumber, totalSteps, message, ...args) => {
    console.log(`${colors.bright}[STEP ${stepNumber}/${totalSteps}]${colors.reset} ${colors.dim}${getTimestamp()}${colors.reset} ${message}`, ...args);
  },

  /**
   * Section header logging (for organizing output)
   */
  section: (title) => {
    const line = '─'.repeat(50);
    console.log(`\n${colors.bright}${colors.cyan}${line}${colors.reset}`);
    console.log(`${colors.bright}${colors.cyan}${title}${colors.reset}`);
    console.log(`${colors.bright}${colors.cyan}${line}${colors.reset}\n`);
  },

  /**
   * Table logging (for structured data)
   */
  table: (data, headers = []) => {
    if (Array.isArray(data) && data.length > 0) {
      console.table(data);
    } else if (typeof data === 'object') {
      console.table(data);
    } else {
      logger.info('Table data:', data);
    }
  },

  /**
   * JSON logging (for complex objects)
   */
  json: (label, data, pretty = true) => {
    if (pretty) {
      console.log(`${colors.cyan}${label}:${colors.reset}`);
      console.log(JSON.stringify(data, null, 2));
    } else {
      console.log(`${colors.cyan}${label}:${colors.reset}`, JSON.stringify(data));
    }
  },

  /**
   * Progress logging (for long operations)
   */
  progress: (current, total, message = '') => {
    const percentage = ((current / total) * 100).toFixed(1);
    const progressBar = '█'.repeat(Math.floor(current / total * 20)) + '░'.repeat(20 - Math.floor(current / total * 20));
    process.stdout.write(`\r${colors.yellow}[PROGRESS]${colors.reset} ${progressBar} ${percentage}% ${message}`);
    
    if (current === total) {
      console.log(); // New line when complete
    }
  },

  /**
   * Performance timing logging
   */
  time: (label) => {
    console.time(`${colors.magenta}[TIMER]${colors.reset} ${label}`);
  },

  timeEnd: (label) => {
    console.timeEnd(`${colors.magenta}[TIMER]${colors.reset} ${label}`);
  },

  /**
   * Raw console access (for special cases)
   */
  raw: console.log,

  /**
   * Clear screen (for interactive experiences)
   */
  clear: () => {
    console.clear();
  }
};

module.exports = { logger, colors };
