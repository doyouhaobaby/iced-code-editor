class MathTools {
  static fibonacci(n) {
    if (n <= 1) {
      return n;
    }
    return MathTools.fibonacci(n - 1) + MathTools.fibonacci(n - 2);
  }

  static factorial(n) {
    let result = 1;
    for (let i = 2; i <= n; i += 1) {
      result *= i;
    }
    return result;
  }

  static isPrime(n) {
    if (n <= 1) {
      return false;
    }
    if (n <= 3) {
      return true;
    }
    if (n % 2 === 0 || n % 3 === 0) {
      return false;
    }
    let i = 5;
    while (i * i <= n) {
      if (n % i === 0 || n % (i + 2) === 0) {
        return false;
      }
      i += 6;
    }
    return true;
  }

  static gcd(a, b) {
    let x = a;
    let y = b;
    while (y !== 0) {
      const r = x % y;
      x = y;
      y = r;
    }
    return x;
  }

  static lcm(a, b) {
    return (a / MathTools.gcd(a, b)) * b;
  }

  static power(base, exp) {
    let result = 1;
    for (let i = 0; i < exp; i += 1) {
      result *= base;
    }
    return result;
  }

  static sqrt(n) {
    return Math.sqrt(n);
  }

  static abs(n) {
    return Math.abs(n);
  }
}

class StringTools {
  static reverse(value) {
    return [...value].reverse().join("");
  }

  static isPalindrome(value) {
    const filtered = [...value].filter((ch) => ch.trim().length > 0).join("");
    return filtered === [...filtered].reverse().join("");
  }

  static capitalize(value) {
    if (!value) {
      return "";
    }
    return value[0].toUpperCase() + value.slice(1).toLowerCase();
  }

  static split(value, delimiter) {
    return value.split(delimiter);
  }

  static trim(value) {
    return value.trim();
  }

  static contains(value, needle) {
    return value.includes(needle);
  }

  static count(value, needle) {
    return value.split(needle).length - 1;
  }
}

class ArrayTools {
  static map(items, fn) {
    return items.map(fn);
  }

  static filter(items, predicate) {
    return items.filter(predicate);
  }

  static reduce(items, fn) {
    if (items.length === 0) {
      return null;
    }
    return items.slice(1).reduce((acc, item) => fn(acc, item), items[0]);
  }

  static find(items, predicate) {
    return items.find(predicate);
  }

  static contains(items, value) {
    return items.includes(value);
  }

  static reverse(items) {
    return [...items].reverse();
  }

  static unique(items) {
    return Array.from(new Set(items));
  }

  static sum(items) {
    return items.reduce((acc, value) => acc + value, 0);
  }
}

const values = [1, 2, 3, 4, 5, 6, 7];
const squares = ArrayTools.map(values, (v) => v * v);
const primes = ArrayTools.filter(values, MathTools.isPrime);
const total = ArrayTools.sum(values);
const greeting = StringTools.capitalize("salut à tous");
const ratio = MathTools.sqrt(42) / 3;
const userMap = Object.fromEntries([
  ["alice", 3],
  ["bob", 5],
  ["carol", 7],
]);
const summary = { squares, primes, total, greeting, ratio, userMap };

export { MathTools, StringTools, ArrayTools, summary };
