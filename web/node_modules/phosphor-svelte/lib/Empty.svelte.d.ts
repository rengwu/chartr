import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EmptyIcon` instead.
 *
 * @example
 * ```svelte
 * <Empty color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Empty: Component<IconComponentProps, {}, "">;
type Empty = ReturnType<typeof Empty>;
export default Empty;
