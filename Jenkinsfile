pipeline {
    agent any

    environment {
        GITHUB_TOKEN = credentials('GITHUB_TOKEN')
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
                sh '''
                curl -L \
                -X POST \
                -H "Accept: application/vnd.github+json" \
                -H "Authorization: Bearer $GITHUB_TOKEN" \
                -H "X-GitHub-Api-Version: 2022-11-28" \
                https://api.github.com/repos/tanndlin/TanScript/statuses/$GIT_COMMIT \
                -d '{"state":"pending","description":"Build in progress","context":"Jenkins"}'
                '''
            }
        }

        stage('Build') {
            steps {
                sh '''
                cd rs
                cargo build --release
                '''
            }
        }

        stage('Lint') {
            steps {
                catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
                    sh '''
                    cd rs
                    cargo fmt -- --check
                    cargo clippy -- -D clippy::pedantic
                    '''
                }
            }
        }

        stage('Test') {
            steps {
                sh '''
                cd rs
                cargo test
                '''
            }
        }
    }

    post {
        success {
            sh '''
            curl -L \
            -X POST \
            -H "Accept: application/vnd.github+json" \
            -H "Authorization: Bearer $GITHUB_TOKEN" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            https://api.github.com/repos/tanndlin/TanScript/statuses/$GIT_COMMIT \
            -d '{"state":"success","description":"Build succeeded","context":"Jenkins"}'
            '''
        }
        failure {
            sh '''
            curl -L \
            -X POST \
            -H "Accept: application/vnd.github+json" \
            -H "Authorization: Bearer $GITHUB_TOKEN" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            https://api.github.com/repos/tanndlin/TanScript/statuses/$GIT_COMMIT \
            -d '{"state":"failure","description":"Build failed","context":"Jenkins"}'
            '''
        }
    }
}
