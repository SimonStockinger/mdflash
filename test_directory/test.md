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
