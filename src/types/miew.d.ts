// The Miew typings omit the event API that the runtime build provides.
export {};

declare module "miew" {
  interface MiewPickEvent {
    obj?: {
      residue?: {
        _sequence: number;
        _type?: { _name: string };
        _chain?: { _name: string };
      };
    };
  }

  interface Miew {
    addEventListener(type: "newpick", listener: (event: MiewPickEvent) => void): void;
  }
}
