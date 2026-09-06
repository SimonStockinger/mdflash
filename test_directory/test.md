---
deck: Rust & Informatik Grundlagen
tags:
    - rust
    - cs
    - networking
date: 2026-09-06
---

# Test Deck: Rust & CS Basics

## 1. Single-Line Cards (::)

Welches Keyword erzwingt in Rust eine unveränderliche Variable?::let
Welcher HTTP-Statuscode bedeutet "Unauthorized"?::401
Was ist die Standard-Gleitkomma-Genauigkeit in Rust?::f64

## 2. Reversed / Bi-directional Cards (:::)

Hauptstadt von Frankreich:::Paris
Kompilierzeit:::Compile time
FIFO (First In First Out):::Queue

## 3. Multi-Line Cards (?)

Welche drei Hauptregeln gelten für Ownership in Rust?
?

1. Jeder Wert in Rust hat genau einen Eigentümer (Owner).
2. Es kann immer nur einen Eigentümer gleichzeitig geben.
3. Wenn der Eigentümer den Scope verlässt, wird der Wert automatisch gedroppt.

Was macht der `?`-Operator in einer Funktion, die `Result<T, E>` zurückgibt?
?

- Entpackt den Wert `Ok(T)`, falls die Operation erfolgreich war.
- Gibt frühzeitig `Err(E)` an den Aufrufer zurück, falls ein Fehler aufgetreten ist.

## 4. Cloze Deletion Cards (==Highlight==)

In Rust werden Daten standardmäßig auf dem ==Stack== abgelegt, während dynamisch wachsende Daten auf dem ==Heap== landen.
Ein ==Mutex== garantiert wechselseitigen Ausschluss bei nebenläufigem Speicherzugriff.
Rust garantiert Speichersicherheit ohne eine ==Garbage Collection== zur Laufzeit.

## 1. Definitionen

Wann heißt eine Funktion $f: \mathbb{R} \to \mathbb{R}$ stetig in $x_0$?
?
$\forall \epsilon > 0 \ \exists \delta > 0 : \forall x \in \mathbb{R} : (|x - x_0| < \delta \implies |f(x) - f(x_0)| < \epsilon)$

Definition der Teilmengenrelation:::$\forall x : (x \in A \implies x \in B) \iff A \subseteq B$

De Morgan'sche Regel (Mengenlehre)::$\overline{A \cup B} = \overline{A} \cap \overline{B}$

## 2. Beweise

Beweis: Die Quadratwurzel $\sqrt{2}$ ist irrational.
?
Beweis durch Widerspruch:

1. Annahme: $\sqrt{2} \in \mathbb{Q}$, also $\sqrt{2} = \frac{p}{q}$ mit teilerfremden $p, q \in \mathbb{Z}$ und $q \neq 0$.
2. Quadrieren: $2 = \frac{p^2}{q^2} \implies 2q^2 = p^2$.
3. Somit ist $p^2$ gerade, woraus folgt, dass $p$ gerade ist: $p = 2k$ für ein $k \in \mathbb{Z}$.
4. Einsetzen: $2q^2 = (2k)^2 = 4k^2 \implies q^2 = 2k^2$.
5. Folglich ist auch $q^2$ und damit $q$ gerade.
6. Widerspruch zur Teilerfremdheit von $p$ und $q$.
   $\implies \sqrt{2} \notin \mathbb{Q} \quad \qed$

## 3. Lückentext (Cloze)

Für alle $n \ge 1$ gilt: $\sum_{i=1}^{n} i = ==\frac{n(n+1)}{2}==$.
