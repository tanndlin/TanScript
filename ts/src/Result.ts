interface IResult<T, E> {
    unwrap(): T;
    map<U>(mapper: (val: T) => U): Result<U, E>;
    expect(str: string): T;
}

export class Ok<T> implements IResult<T, never> {
    public readonly ok: true = true;
    constructor(public val: T) {}

    unwrap(): T {
        return this.val;
    }

    map<T2>(mapper: (val: T) => T2): Ok<T2> {
        return new Ok(mapper(this.val));
    }

    expect(_str: string) {
        return this.val;
    }
}

export class Err<E> implements IResult<never, E> {
    public readonly ok: false = false;
    constructor(public val: E) {}

    unwrap(): never {
        throw new Error('Tried to unwrap Error');
    }

    map(_mapper: unknown): Err<E> {
        return this;
    }

    expect(str: string): never {
        throw new Error(str);
    }
}

export type Result<T, X> = Ok<T> | Err<X>;
