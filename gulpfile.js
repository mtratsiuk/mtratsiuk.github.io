'use strict';

var gulp = require('gulp'),
    concat = require('gulp-concat'),
    watch = require('gulp-watch'),
    prefixer = require('gulp-autoprefixer'),
    uglify = require('gulp-uglify'),
    sass = require('gulp-sass'),
    sourcemaps = require('gulp-sourcemaps'),
    cssmin = require('gulp-minify-css'),
    htmlmin = require('gulp-minify-html'),
    rimraf = require('rimraf'),
    browserSync = require("browser-sync"),
    streamqueue = require('streamqueue'),
    gulpSequence = require('gulp-sequence'),
    reload = browserSync.reload;

var path = {
    build: {
        html: './',
        css: 'build/css/',
        img: 'build/img/',
        fonts: 'build/fonts/'
    },
    src: {
        html: 'src/*.html',
        style: 'src/*.scss',
        img: 'src/img/**/*.*',
        fonts: 'src/fonts/**/*.*'
    },
    watch: {
        html: 'src/*.html',
        style: 'src/*.scss',
        img: 'src/img/**/*.*',
        fonts: 'src/fonts/**/*.*'
    },
    clean: './build'
};

var config = {
    server: {
        baseDir: "./"
    },
    host: 'localhost',
    port: 9000
};

gulp.task('webserver', function () {
    browserSync(config);
});

gulp.task('clean', function (cb) {
    rimraf(path.clean, cb);
});

gulp.task('html:build', function () {
    gulp.src(path.src.html)
        .pipe(htmlmin())
        .pipe(gulp.dest(path.build.html))
        .pipe(reload({ stream: true }));
});

gulp.task('style:build', function () {
    var libs = gulp.src([
        'src/normalize.css'
    ]);

    streamqueue.obj(libs,
        gulp.src(path.src.style)
            .pipe(sourcemaps.init())
            .pipe(sass({
                includePaths: ['src/style/'],
                outputStyle: 'compressed',
                sourceMap: true,
                errLogToConsole: true
            }))
            .pipe(prefixer())
        )
        .pipe(concat('style.css'))
        .pipe(cssmin())
        .pipe(sourcemaps.write())
        .pipe(gulp.dest(path.build.css))
        .pipe(reload({ stream: true }));
});

gulp.task('image:build', function () {
    gulp.src(path.src.img)
        .pipe(gulp.dest(path.build.img))
        .pipe(reload({ stream: true }));
});

gulp.task('fonts:build', function () {
    gulp.src(path.src.fonts)
        .pipe(gulp.dest(path.build.fonts));
});

gulp.task('build', gulpSequence('clean',
    'style:build',
    ['html:build',
        'fonts:build',
        'image:build']));

gulp.task('watch', function () {
    watch([path.watch.html], function (event, cb) {
        gulp.start('html:build');
    });
    watch([path.watch.style], function (event, cb) {
        gulp.start('style:build');
    });
    watch([path.watch.img], function (event, cb) {
        gulp.start('image:build');
    });
    watch([path.watch.fonts], function (event, cb) {
        gulp.start('fonts:build');
    });
});

gulp.task('default', gulpSequence('build', 'webserver', 'watch'));