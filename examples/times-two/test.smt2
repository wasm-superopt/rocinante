(declare-const x (_ BitVec 32))

(assert (not (= (bvshl x (_ bv1 32)) (bvadd x x))))

(check-sat)  ; check satisfiability
(get-model)  ; show an assignment satisfying the formula (if any)
