import PageController from './controllers/PageController.js';
import { PAGES } from './constants.js';
import InitialPage from './pages/InitialPage.js';
import SetupPage from './pages/SetupPage.js';
import HomePage from './pages/HomePage.js';

class Main {
    constructor() {
        this.init();
    }

    init() {
        console.log('App initialized');
        this.registerPages();
        PageController.showPage(PAGES.INITIAL);
    }

    registerPages() {
        // Register all pages with the PageController
        PageController.register(new InitialPage(), PAGES.INITIAL);
        PageController.register(new SetupPage(), PAGES.SETUP);
        PageController.register(new HomePage(), PAGES.HOME);
    }
}

let app = new Main();
export default app;
//fetch('/api/health').then(r => r.text()).then(t => {
//    document.getElementById('health')!.textContent = t;
//});  