from globalopt import bayes1, furasn


def main() -> None:
    a = [-0.25, -0.125]
    b = [0.5, 0.625]

    result = bayes1(a, b, evaluations=200, initial_points=20, objective=furasn)
    print(f"BAYES1 best f: {result.best_f:.8f}")
    print(f"BAYES1 best x: {result.best_x}")
    print(f"BAYES1 best iteration: {result.best_iter}")


if __name__ == "__main__":
    main()
