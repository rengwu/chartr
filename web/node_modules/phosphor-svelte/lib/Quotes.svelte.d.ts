import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `QuotesIcon` instead.
 *
 * @example
 * ```svelte
 * <Quotes color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Quotes: Component<IconComponentProps, {}, "">;
type Quotes = ReturnType<typeof Quotes>;
export default Quotes;
