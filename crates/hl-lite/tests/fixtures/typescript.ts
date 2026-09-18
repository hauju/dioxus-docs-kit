// Types and helpers for the generated OpenAPI client.
import type { Operation, Schema } from "./openapi";

export type HttpMethod = "get" | "post" | "put" | "patch" | "delete";

export interface Endpoint<T = unknown> {
    readonly operationId: string;
    readonly method: HttpMethod;
    readonly path: `/${string}`;
    summary?: string;
    tags: string[];
    response: T;
}

export enum Status {
    Draft = "draft",
    Published = "published",
    Archived = "archived",
}

type Awaitable<T> = T | Promise<T>;
type KeysOf<T> = keyof T & string;

const DEFAULT_TIMEOUT = 10_000;

export abstract class Client {
    protected constructor(
        private readonly baseUrl: string,
        private timeout: number = DEFAULT_TIMEOUT,
    ) {}

    abstract authorize(): Awaitable<Record<string, string>>;

    async call<T>(endpoint: Endpoint<T>, body?: unknown): Promise<T> {
        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), this.timeout);
        try {
            const headers = await this.authorize();
            const res = await fetch(this.baseUrl + endpoint.path, {
                method: endpoint.method.toUpperCase(),
                headers: { "content-type": "application/json", ...headers },
                body: body === undefined ? null : JSON.stringify(body),
                signal: controller.signal,
            });
            if (!res.ok) throw new ApiError(res.status, endpoint.operationId);
            return (await res.json()) as T;
        } finally {
            clearTimeout(timer);
        }
    }
}

export class ApiError extends Error {
    constructor(
        public readonly status: number,
        public readonly operationId: string,
    ) {
        super(`${operationId} failed with ${status}`);
        this.name = "ApiError";
    }
}

export function isEndpoint(value: unknown): value is Endpoint {
    return typeof value === "object" && value !== null && "operationId" in value;
}

export const slugify = (input: string): string =>
    input
        .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
        .replace(/[^a-zA-Z0-9]+/g, "-")
        .toLowerCase();

export type OperationMap = Record<KeysOf<Schema>, Operation>;
