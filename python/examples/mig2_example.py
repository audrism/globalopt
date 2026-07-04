from globalopt import furasn, mig2


def main() -> None:
    a = [-0.25, -0.125]
    b = [0.5, 0.625]

    result = mig2(a, b, evaluations=200, objective=furasn)
    print(f"MIG2 best f: {result.best_f:.8f}")
    print(f"MIG2 best x: {result.best_x}")
    print(f"MIG2 best iteration: {result.best_iter}")


if __name__ == "__main__":
    main()
