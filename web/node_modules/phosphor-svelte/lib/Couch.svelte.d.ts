import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CouchIcon` instead.
 *
 * @example
 * ```svelte
 * <Couch color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Couch: Component<IconComponentProps, {}, "">;
type Couch = ReturnType<typeof Couch>;
export default Couch;
