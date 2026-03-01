type Predicate<T> = (value: T) => boolean;
type Mapper<T, U> = (value: T) => U;
type Reducer<T> = (left: T, right: T) => T;

export class MathTools {
  static fibonacci(n: number): number {
    if (n <= 1) {
      return n;
    }
    return MathTools.fibonacci(n - 1) + MathTools.fibonacci(n - 2);
  }

  static factorial(n: number): number {
    let result = 1;
    for (let i = 2; i <= n; i += 1) {
      result *= i;
    }
    return result;
  }

  static isPrime(n: number): boolean {
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

  static gcd(a: number, b: number): number {
    let x = a;
    let y = b;
    while (y !== 0) {
      const r = x % y;
      x = y;
      y = r;
    }
    return x;
  }

  static lcm(a: number, b: number): number {
    return (a / MathTools.gcd(a, b)) * b;
  }

  static power(base: number, exp: number): number {
    let result = 1;
    for (let i = 0; i < exp; i += 1) {
      result *= base;
    }
    return result;
  }

  static sqrt(n: number): number {
    return Math.sqrt(n);
  }

  static abs(n: number): number {
    return Math.abs(n);
  }
}

export class StringTools {
  static reverse(value: string): string {
    return [...value].reverse().join("");
  }

  static isPalindrome(value: string): boolean {
    const filtered = [...value].filter((ch) => ch.trim().length > 0).join("");
    return filtered === [...filtered].reverse().join("");
  }

  static capitalize(value: string): string {
    if (!value) {
      return "";
    }
    return value[0].toUpperCase() + value.slice(1).toLowerCase();
  }

  static split(value: string, delimiter: string): string[] {
    return value.split(delimiter);
  }

  static trim(value: string): string {
    return value.trim();
  }

  static contains(value: string, needle: string): boolean {
    return value.includes(needle);
  }

  static count(value: string, needle: string): number {
    return value.split(needle).length - 1;
  }
}

export class ArrayTools {
  static map<T, U>(items: T[], fn: Mapper<T, U>): U[] {
    return items.map(fn);
  }

  static filter<T>(items: T[], predicate: Predicate<T>): T[] {
    return items.filter(predicate);
  }

  static reduce<T>(items: T[], fn: Reducer<T>): T | null {
    if (items.length === 0) {
      return null;
    }
    return items.slice(1).reduce((acc, item) => fn(acc, item), items[0]);
  }

  static find<T>(items: T[], predicate: Predicate<T>): T | undefined {
    return items.find(predicate);
  }

  static contains<T>(items: T[], value: T): boolean {
    return items.includes(value);
  }

  static reverse<T>(items: T[]): T[] {
    return [...items].reverse();
  }

  static unique<T>(items: T[]): T[] {
    return Array.from(new Set(items));
  }

  static sum(items: number[]): number {
    return items.reduce((acc, value) => acc + value, 0);
  }
}

export interface User {
  name: string;
  score: number;
}

export function buildUserMap(users: User[]): Record<string, number> {
  return Object.fromEntries(users.map((user) => [user.name, user.score]));
}

const values = [1, 2, 3, 4, 5, 6, 7];
const squares = ArrayTools.map(values, (v) => v * v);
const primes = ArrayTools.filter(values, MathTools.isPrime);
const total = ArrayTools.sum(values);
const greeting = StringTools.capitalize("salut à tous");
const ratio = MathTools.sqrt(42) / 3;
const userMap = buildUserMap([
  { name: "alice", score: 3 },
  { name: "bob", score: 5 },
  { name: "carol", score: 7 },
]);
const summary = { squares, primes, total, greeting, ratio, userMap };
export { summary };
