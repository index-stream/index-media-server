import Page from './Page.js';
import { PAGES } from '../constants.js';
import PageController from '../controllers/PageController.js';
import Backend from '../clients/Backend.js';
import Config from '../models/Config.js';

export default class InitialPage extends Page {
    constructor() {
        super(PAGES.INITIAL);
    }

    async onShow() {
        console.log('Initial page shown');
        
        const isAlreadySetup = await this.checkIfSetup();
        if(isAlreadySetup) {
            //TODO: create HOME page
            PageController.showPage(PAGES.HOME);
        } else {
            PageController.showPage(PAGES.SETUP);
        }
    }

    async checkIfSetup() {
        try {
            const response = await Backend.getConfiguration();
            console.log('Configuration:', response.config);
            Config.setConfig(response.config);
            
            return !!response.config;
        } catch (error) {
            return false;
        }
    }
}
