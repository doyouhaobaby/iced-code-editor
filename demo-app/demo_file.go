package main

import (
	"fmt"
	"math"
	"sort"
	"strings"
)

type MathTools struct{}

func (MathTools) Fibonacci(n int) int {
	if n <= 1 {
		return n
	}
	return MathTools{}.Fibonacci(n-1) + MathTools{}.Fibonacci(n-2)
}

func (MathTools) Factorial(n int) int {
	result := 1
	for i := 2; i <= n; i++ {
		result *= i
	}
	return result
}

func (MathTools) IsPrime(n int) bool {
	if n <= 1 {
		return false
	}
	if n <= 3 {
		return true
	}
	if n%2 == 0 || n%3 == 0 {
		return false
	}
	for i := 5; i*i <= n; i += 6 {
		if n%i == 0 || n%(i+2) == 0 {
			return false
		}
	}
	return true
}

func (MathTools) Gcd(a, b int) int {
	x := a
	y := b
	for y != 0 {
		x, y = y, x%y
	}
	return x
}

func (m MathTools) Lcm(a, b int) int {
	return a / m.Gcd(a, b) * b
}

func (MathTools) Power(base, exp int) int {
	result := 1
	for i := 0; i < exp; i++ {
		result *= base
	}
	return result
}

func (MathTools) Sqrt(n float64) float64 {
	return math.Sqrt(n)
}

func (MathTools) Abs(n int) int {
	return int(math.Abs(float64(n)))
}

type StringTools struct{}

func (StringTools) Reverse(value string) string {
	runes := []rune(value)
	for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
		runes[i], runes[j] = runes[j], runes[i]
	}
	return string(runes)
}

func (s StringTools) IsPalindrome(value string) bool {
	normalized := strings.ReplaceAll(value, " ", "")
	return normalized == s.Reverse(normalized)
}

func (StringTools) Capitalize(value string) string {
	if value == "" {
		return ""
	}
	runes := []rune(value)
	first := strings.ToUpper(string(runes[0]))
	rest := strings.ToLower(string(runes[1:]))
	return first + rest
}

func (StringTools) Split(value, delimiter string) []string {
	return strings.Split(value, delimiter)
}

func (StringTools) Trim(value string) string {
	return strings.TrimSpace(value)
}

func (StringTools) Contains(value, needle string) bool {
	return strings.Contains(value, needle)
}

func (StringTools) Count(value, needle string) int {
	return strings.Count(value, needle)
}

type ArrayTools struct{}

func (ArrayTools) Map(items []int, fn func(int) int) []int {
	result := make([]int, len(items))
	for i, v := range items {
		result[i] = fn(v)
	}
	return result
}

func (ArrayTools) Filter(items []int, predicate func(int) bool) []int {
	var result []int
	for _, v := range items {
		if predicate(v) {
			result = append(result, v)
		}
	}
	return result
}

func (ArrayTools) Reduce(items []int, fn func(int, int) int) *int {
	if len(items) == 0 {
		return nil
	}
	acc := items[0]
	for _, v := range items[1:] {
		acc = fn(acc, v)
	}
	return &acc
}

func (ArrayTools) Find(items []int, predicate func(int) bool) *int {
	for _, v := range items {
		if predicate(v) {
			value := v
			return &value
		}
	}
	return nil
}

func (ArrayTools) Contains(items []int, value int) bool {
	for _, v := range items {
		if v == value {
			return true
		}
	}
	return false
}

func (ArrayTools) Reverse(items []int) []int {
	result := append([]int(nil), items...)
	for i, j := 0, len(result)-1; i < j; i, j = i+1, j-1 {
		result[i], result[j] = result[j], result[i]
	}
	return result
}

func (ArrayTools) Unique(items []int) []int {
	seen := map[int]struct{}{}
	var result []int
	for _, v := range items {
		if _, ok := seen[v]; !ok {
			seen[v] = struct{}{}
			result = append(result, v)
		}
	}
	return result
}

func (ArrayTools) Sum(items []int) int {
	sum := 0
	for _, v := range items {
		sum += v
	}
	return sum
}

type User struct {
	Name  string
	Score int
}

func BuildUserMap(users []User) map[string]int {
	result := make(map[string]int)
	for _, user := range users {
		result[user.Name] = user.Score
	}
	return result
}

func main() {
	values := []int{1, 2, 3, 4, 5, 6, 7}
	mathTools := MathTools{}
	stringTools := StringTools{}
	arrayTools := ArrayTools{}

	squares := arrayTools.Map(values, func(v int) int { return v * v })
	primes := arrayTools.Filter(values, mathTools.IsPrime)
	total := arrayTools.Sum(values)
	greeting := stringTools.Capitalize("salut à tous")
	ratio := mathTools.Sqrt(42) / 3
	users := []User{{Name: "alice", Score: 3}, {Name: "bob", Score: 5}}
	userMap := BuildUserMap(users)

	summary := map[string]any{
		"squares":  squares,
		"primes":   primes,
		"total":    total,
		"greeting": greeting,
		"ratio":    ratio,
		"userMap":  userMap,
	}

	keys := make([]string, 0, len(summary))
	for key := range summary {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	for _, key := range keys {
		fmt.Println(key, summary[key])
	}
}
