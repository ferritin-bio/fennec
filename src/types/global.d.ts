declare global {
  interface Window {
    molstar_viewer: any; // or more specific type if available from molstar
    miew_viewer: any; // or more specific type if available from molstar
  }
}

export {};
