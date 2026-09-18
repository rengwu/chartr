import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TestTubeIcon` instead.
 *
 * @example
 * ```svelte
 * <TestTube color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TestTube: Component<IconComponentProps, {}, "">;
type TestTube = ReturnType<typeof TestTube>;
export default TestTube;
